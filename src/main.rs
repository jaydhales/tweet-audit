mod config;
mod gemini;
mod routes;
mod state;
mod tweets;

use crate::config::load_config;
use crate::gemini::{
    GeminiClient,
};
use crate::routes::{health_check, run_audit};
use crate::state::AppState;
use axum::routing::post;
use axum::{Router, routing::get};
use std::sync::Arc;
use tweet_audit::error::AppError;
use tweet_audit::telemetry::init_tracing;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Setup Logger
    init_tracing();

    //Load Config
    let config = load_config()?;

    // Load AI Client
    let client = GeminiClient::new()?;
 
    let app_state = Arc::new(AppState { client, config });

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/health", get(health_check))
        .route("/audit", post(run_audit))
        .with_state(app_state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .map_err(|e| AppError::Io(e))?;

    tracing::info!("🚀 Server running on port 3000");
    axum::serve(listener, app).await?;

    Ok(())
}

// GET /
// GET /health
// GET /config
// POST /audit
// GET /audit/results
// GET /audit/results.csv
