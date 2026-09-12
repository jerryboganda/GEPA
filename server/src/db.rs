//! Postgres persistence for GEPA v2 (DECISIONS.md D-021 — replaces the
//! earlier Firestore design; product owner directive: Postgres + Rust only,
//! no Firebase).
//!
//! Every table used by `repos.rs` (`sessions`, `jobs`, `review_queue`) has
//! the same shape — `id TEXT PRIMARY KEY, data JSONB, version BIGINT` — so
//! this client exposes one small get/create/set/delete/list API instead of
//! bespoke queries per table. `shared_engine`/`state` domain structs are
//! serialized to/from `data` wholesale via `serde_json`, unchanged from the
//! earlier Firestore design (`SessionRepo` etc. barely changed — only what
//! they're backed by did).
//!
//! Plain runtime SQL via `tokio-postgres` (no query macros, no ORM): this
//! machine's Rust builds are slow, and a compile-time-checked-query crate
//! (e.g. `sqlx`'s macros) needs a live DB at build time anyway, which the
//! product owner explicitly does not want on this machine — everything
//! DB-dependent is verified in GitHub Actions instead (see `.github/workflows/ci.yml`).

#![allow(dead_code)]

use deadpool_postgres::{Config, Pool, Runtime};
use serde_json::Value;
use std::fmt;
use std::sync::Arc;
use tokio_postgres::NoTls;

#[derive(Debug)]
pub enum DbError {
    NotFound,
    Conflict,
    Query(String),
    Pool(String),
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbError::NotFound => write!(f, "row not found"),
            DbError::Conflict => write!(f, "update precondition failed (concurrent write)"),
            DbError::Query(e) => write!(f, "postgres query error: {e}"),
            DbError::Pool(e) => write!(f, "postgres pool error: {e}"),
        }
    }
}

impl std::error::Error for DbError {}

/// The only tables this client ever touches — table names are always one of
/// these hardcoded literals from `repos.rs`, never user input, but this
/// allowlist is cheap defense-in-depth against SQL injection via a future
/// mistaken call site.
fn validate_table(table: &str) -> Result<(), DbError> {
    match table {
        "sessions" | "jobs" | "review_queue" => Ok(()),
        other => Err(DbError::Query(format!("unknown table: {other}"))),
    }
}

pub struct FetchedRow {
    pub data: Value,
    pub version: i64,
}

pub struct PostgresClient {
    pool: Pool,
}

impl PostgresClient {
    /// Builds the pool and runs migrations. Never panics on a bad/missing
    /// `DATABASE_URL` — the server still boots (and serves `/healthz` as
    /// degraded) so a misconfigured DB doesn't take down static asset
    /// serving or the pure seed-bank-only endpoints.
    pub async fn from_env() -> Arc<Self> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://gepa:gepa@localhost:5432/gepa".to_string());
        let mut cfg = Config::new();
        cfg.url = Some(database_url);
        let pool = match cfg.create_pool(Some(Runtime::Tokio1), NoTls) {
            Ok(p) => p,
            Err(_) => {
                // The error Display for a config-parse failure can embed the
                // DSN (which contains credentials) — never log it raw
                // (AGENTS.md logging redaction list, 09 §2). The only
                // failure mode here is a malformed DATABASE_URL.
                tracing::error!("failed to build Postgres pool — DATABASE_URL is malformed (value redacted)");
                cfg.url = Some("postgres://invalid/invalid".to_string());
                cfg.create_pool(Some(Runtime::Tokio1), NoTls)
                    .expect("deadpool pool construction is infallible for a well-formed URL string")
            }
        };
        let client = Arc::new(Self { pool });
        if let Err(e) = client.run_migrations().await {
            tracing::warn!("Postgres migrations did not run (DB may be unreachable yet): {e}");
        }
        client
    }

    async fn run_migrations(&self) -> Result<(), DbError> {
        let conn = self.pool.get().await.map_err(|e| DbError::Pool(e.to_string()))?;
        conn.batch_execute(include_str!("../migrations/001_init.sql"))
            .await
            .map_err(|e| DbError::Query(e.to_string()))
    }

    /// Raw pool access for the one table that isn't a generic JSONB
    /// document (`staff_users` — see `api.rs::login`).
    pub fn pool(&self) -> &Pool {
        &self.pool
    }

    /// Cheap reachability check for `/healthz`.
    pub async fn ping(&self) -> bool {
        match self.pool.get().await {
            Ok(conn) => conn.simple_query("SELECT 1").await.is_ok(),
            Err(_) => false,
        }
    }

    pub async fn get(&self, table: &str, id: &str) -> Result<Option<FetchedRow>, DbError> {
        validate_table(table)?;
        let conn = self.pool.get().await.map_err(|e| DbError::Pool(e.to_string()))?;
        let sql = format!("SELECT data, version FROM {table} WHERE id = $1");
        let row = conn
            .query_opt(&sql, &[&id])
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;
        Ok(row.map(|r| FetchedRow { data: r.get(0), version: r.get(1) }))
    }

    /// Insert a brand-new row. `Err(Conflict)` if `id` already exists.
    pub async fn create(&self, table: &str, id: &str, value: &Value) -> Result<(), DbError> {
        validate_table(table)?;
        let conn = self.pool.get().await.map_err(|e| DbError::Pool(e.to_string()))?;
        let sql = format!("INSERT INTO {table} (id, data, version) VALUES ($1, $2, 1)");
        conn.execute(&sql, &[&id, value])
            .await
            .map(|_| ())
            .map_err(|e| {
                if e.code() == Some(&tokio_postgres::error::SqlState::UNIQUE_VIOLATION) {
                    DbError::Conflict
                } else {
                    DbError::Query(e.to_string())
                }
            })
    }

    /// Full-row overwrite with an optimistic-concurrency precondition:
    /// succeeds only if the row's `version` still matches `expected_version`
    /// (from a prior `get`), then bumps `version` by one. `Err(Conflict)`
    /// means another request modified the row in between.
    pub async fn set(&self, table: &str, id: &str, value: &Value, expected_version: i64) -> Result<(), DbError> {
        validate_table(table)?;
        let conn = self.pool.get().await.map_err(|e| DbError::Pool(e.to_string()))?;
        let sql = format!("UPDATE {table} SET data = $1, version = version + 1 WHERE id = $2 AND version = $3");
        let rows = conn
            .execute(&sql, &[value, &id, &expected_version])
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;
        if rows == 0 {
            // Either the row doesn't exist, or the version precondition failed.
            return Err(DbError::Conflict);
        }
        Ok(())
    }

    pub async fn delete(&self, table: &str, id: &str) -> Result<bool, DbError> {
        validate_table(table)?;
        let conn = self.pool.get().await.map_err(|e| DbError::Pool(e.to_string()))?;
        let sql = format!("DELETE FROM {table} WHERE id = $1");
        let rows = conn.execute(&sql, &[&id]).await.map_err(|e| DbError::Query(e.to_string()))?;
        Ok(rows > 0)
    }

    /// Every row in a table. Used only by admin reports and the retention
    /// job, which need every document anyway.
    /// # ponytail: full-table scan, fine at pilot volume — add a real
    /// indexed query (or paginate) if row counts grow past the tens of
    /// thousands.
    pub async fn list(&self, table: &str) -> Result<Vec<(String, Value)>, DbError> {
        validate_table(table)?;
        let conn = self.pool.get().await.map_err(|e| DbError::Pool(e.to_string()))?;
        let sql = format!("SELECT id, data FROM {table}");
        let rows = conn.query(&sql, &[]).await.map_err(|e| DbError::Query(e.to_string()))?;
        Ok(rows.into_iter().map(|r| (r.get(0), r.get(1))).collect())
    }
}
