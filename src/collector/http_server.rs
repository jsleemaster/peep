use axum::{extract::State, http::StatusCode, routing::{get, post}, Json, Router};
use serde_json::Value;
use tokio::sync::mpsc;
use tower_http::limit::RequestBodyLimitLayer;

use crate::protocol::hook_payload::parse_hook_payload;
use crate::protocol::types::RawIngestEvent;

#[derive(Clone)]
struct AppState {
    tx: mpsc::Sender<RawIngestEvent>,
}

async fn hook_handler(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    if let Some(event) = parse_hook_payload(&body) {
        if let Err(_e) = state.tx.try_send(event) {
            tracing::warn!("Event channel full, dropping event");
        }
    }
    (
        StatusCode::OK,
        Json(serde_json::json!({"status": "ok"})),
    )
}

async fn health_handler() -> (StatusCode, &'static str) {
    (StatusCode::OK, "ok")
}

/// Start the HTTP hook server. Returns when the server shuts down.
pub async fn run_http_server(
    bind: String,
    port: u16,
    tx: mpsc::Sender<RawIngestEvent>,
) -> anyhow::Result<()> {
    let state = AppState { tx };
    let app = Router::new()
        .route("/hook", post(hook_handler))
        .route("/health", get(health_handler))
        .layer(RequestBodyLimitLayer::new(256 * 1024)) // 256KB max
        .with_state(state);

    let addr = format!("{}:{}", bind, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
