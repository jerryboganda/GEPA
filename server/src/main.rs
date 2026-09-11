mod ai;
mod api;
mod repos;
mod services;
mod state;

use ai::GeminiClient;
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use parking_lot::RwLock;
use repos::SeedBank;
use state::{AppState, SharedState};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "server=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Initializing GEPA v2 Backend & Assessment Engine Server...");

    let seed_path = std::env::var("SEED_DIR").unwrap_or_else(|_| "seed".to_string());
    tracing::info!("Loading seed item bank from '{}'...", seed_path);
    let seed_bank = SeedBank::load_from_dir(&seed_path)
        .expect("Failed to load GEPA seed bank from seed directory");

    tracing::info!(
        "Seed bank loaded successfully: {} LS items, {} RD items, {} LSN items, {} SPK tasks, {} WRT tasks, {} keys",
        seed_bank.ls_items.len(),
        seed_bank.rd_items.len(),
        seed_bank.lsn_items.len(),
        seed_bank.speaking_tasks.len(),
        seed_bank.writing_tasks.len(),
        seed_bank.restricted_keys.len(),
    );

    let gemini_client = Arc::new(GeminiClient::new());

    let state: SharedState = Arc::new(AppState {
        seed_bank: RwLock::new(seed_bank),
        sessions: RwLock::new(std::collections::HashMap::new()),
        review_queue: RwLock::new(Vec::new()),
        jobs: RwLock::new(std::collections::HashMap::new()),
        idempotency_cache: RwLock::new(std::collections::HashMap::new()),
        gemini_client,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/healthz", get(api::healthz))
        .route("/api/sessions", post(api::create_session))
        .route("/api/sessions/:id", get(api::get_session_state))
        .route("/api/sessions/:id", delete(api::delete_session))
        .route(
            "/api/sessions/:id/modules/:module/start",
            post(api::start_module),
        )
        .route(
            "/api/sessions/:id/responses",
            post(api::submit_objective_response),
        )
        .route(
            "/api/sessions/:id/results/receptive",
            get(api::get_receptive_result),
        )
        .route(
            "/api/sessions/:id/speaking/start",
            get(api::start_speaking),
        )
        .route(
            "/api/sessions/:id/speaking/:task_id/submit",
            post(api::submit_speaking),
        )
        .route(
            "/api/sessions/:id/writing/start",
            get(api::start_writing),
        )
        .route(
            "/api/sessions/:id/writing/:task_id/draft",
            put(api::save_writing_draft),
        )
        .route(
            "/api/sessions/:id/writing/:task_id/submit",
            post(api::submit_writing),
        )
        .route(
            "/api/sessions/:id/results/status",
            get(api::get_results_status),
        )
        .route(
            "/api/sessions/:id/results/full",
            get(api::get_full_results),
        )
        .route("/api/media/audio/:file", get(api::serve_audio))
        .route("/api/media/stimuli/:stimulus_id/url", get(api::get_stimulus_audio_url))
        .route("/api/media/responses/:task_id/upload-url", post(api::get_upload_url))
        .route("/api/media/upload", post(api::upload_media))
        .route("/api/review/queue", get(api::review_queue))
        .route("/api/review/sessions/:id", get(api::review_session))
        .route("/api/review/sessions/:id/rescore", post(api::rescore_session))
        .route("/api/review/sessions/:id/human-score", post(api::human_score_session))
        .route("/api/admin/forms/check", post(api::admin_forms_check))
        .route("/api/admin/reports/exposure", get(api::admin_exposure_report))
        .route("/api/admin/reports/telemetry", get(api::admin_telemetry_report))
        .route("/api/admin/reports/metrics", get(api::admin_metrics_report))
        .route("/api/admin/jobs", get(api::admin_jobs_report))
        .route("/api/admin/seed/load", post(api::admin_seed_load))
        .route("/api/admin/audio/produce", post(api::admin_audio_produce))
        .route("/api/admin/retention/run", post(api::admin_retention_run))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let client_dist = std::env::var("CLIENT_DIST").unwrap_or_else(|_| "client/dist".to_string());
    let fallback_path = format!("{}/index.html", client_dist);
    let serve_dir = tower_http::services::ServeDir::new(&client_dist)
        .fallback(tower_http::services::ServeFile::new(fallback_path));
    let app = app.fallback_service(serve_dir);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("GEPA Backend listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
