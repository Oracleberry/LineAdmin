pub mod line_webhook;
pub mod line_client;

use axum::{
    routing::{get, post},
    Router,
};
use sqlx::SqlitePool;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
}

pub fn create_router(db: SqlitePool) -> Router {
    let state = Arc::new(AppState { db });

    Router::new()
        .route("/", get(health_check))
        .route("/webhook/line", post(line_webhook::handle_webhook))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn health_check() -> &'static str {
    "OK"
}
