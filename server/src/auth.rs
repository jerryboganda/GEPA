//! Self-issued JWT auth for GEPA v2 (DECISIONS.md D-021 — replaces the
//! earlier Firebase Auth design; product owner directive: no Firebase).
//!
//! Candidates need no account at all: `POST /api/sessions` mints a fresh
//! anonymous `candidate_uid` and signs a token for it server-side (mirrors
//! `01_PRD.md §2`'s "no identity verification in free beta", just without an
//! external identity provider issuing the token). Reviewers/admins log in
//! with email + password against the `staff_users` Postgres table
//! (`POST /api/auth/login`, see `api.rs`); their token carries a `role`
//! claim the server checks on every reviewer/admin route.
//!
//! HS256 with a shared server secret — no JWKS fetching, no emulator-vs-prod
//! branching (that complexity was specific to verifying tokens issued by an
//! external provider; self-issued tokens don't need it).

#![allow(dead_code)]

use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::{request::Parts, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::Duration;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

use crate::state::SharedState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// `candidate_uid` for candidates, `staff_users.id` for reviewer/admin.
    pub sub: String,
    /// `None` for candidates; `Some("reviewer" | "admin")` for staff.
    #[serde(default)]
    pub role: Option<String>,
    pub exp: usize,
}

pub struct JwtService {
    secret: String,
}

impl JwtService {
    pub fn from_env() -> Arc<Self> {
        let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
            tracing::warn!(
                "JWT_SECRET not set — using an insecure development default. \
                 Set a real random secret (env JWT_SECRET) before deploying."
            );
            "gepa-dev-insecure-secret-change-me".to_string()
        });
        Arc::new(Self { secret })
    }

    pub fn issue(&self, sub: &str, role: Option<&str>, ttl: Duration) -> Result<String, jsonwebtoken::errors::Error> {
        let exp = (chrono::Utc::now() + ttl).timestamp() as usize;
        let claims = Claims { sub: sub.to_string(), role: role.map(|s| s.to_string()), exp };
        encode(&Header::default(), &claims, &EncodingKey::from_secret(self.secret.as_bytes()))
    }

    pub fn verify(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )?;
        Ok(data.claims)
    }
}

/// Candidate tokens don't expire during a normal session; staff tokens are
/// short-lived (re-login rather than a silent-refresh flow — simplest thing
/// that works for a small reviewer/admin user base).
pub const CANDIDATE_TOKEN_TTL_DAYS: i64 = 30;
pub const STAFF_TOKEN_TTL_HOURS: i64 = 12;

pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| e.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    match PasswordHash::new(hash) {
        Ok(parsed) => Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok(),
        Err(_) => false,
    }
}

fn unauthorized(message: &str) -> Response {
    (StatusCode::UNAUTHORIZED, Json(json!({ "error": "unauthorized", "message": message }))).into_response()
}

fn forbidden(message: &str) -> Response {
    (StatusCode::FORBIDDEN, Json(json!({ "error": "forbidden", "message": message }))).into_response()
}

/// Any candidate/reviewer/admin with a valid token.
pub struct AuthUser {
    pub uid: String,
    pub role: Option<String>,
}

#[async_trait]
impl FromRequestParts<SharedState> for AuthUser {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &SharedState) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| unauthorized("missing Authorization header"))?;
        let token = header
            .strip_prefix("Bearer ")
            .ok_or_else(|| unauthorized("Authorization header must be a Bearer token"))?;
        let claims = state.jwt.verify(token).map_err(|e| unauthorized(&e.to_string()))?;
        Ok(AuthUser { uid: claims.sub, role: claims.role })
    }
}

/// A valid token with role `reviewer` or `admin`.
pub struct StaffAuth(pub AuthUser);

#[async_trait]
impl FromRequestParts<SharedState> for StaffAuth {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &SharedState) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, state).await?;
        match user.role.as_deref() {
            Some("reviewer") | Some("admin") => Ok(StaffAuth(user)),
            _ => Err(forbidden("reviewer or admin role required")),
        }
    }
}

/// A valid token with role `admin`.
pub struct AdminAuth(pub AuthUser);

#[async_trait]
impl FromRequestParts<SharedState> for AdminAuth {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &SharedState) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, state).await?;
        match user.role.as_deref() {
            Some("admin") => Ok(AdminAuth(user)),
            _ => Err(forbidden("admin role required")),
        }
    }
}

/// `true` if the caller owns the session or is staff. Call right after
/// fetching the session document in a handler — not threaded through
/// `services.rs`, since this is a per-request authorization check, not
/// business logic. A plain bool, not `Result<(), Response>`: every call site
/// builds its own response on failure anyway, and `Response` is 128+ bytes —
/// too large for a `Result` error variant on a function called on every
/// session-scoped request.
pub fn assert_owns_or_staff(candidate_uid: &str, user: &AuthUser) -> bool {
    let is_staff = matches!(user.role.as_deref(), Some("reviewer") | Some("admin"));
    is_staff || candidate_uid == user.uid
}
