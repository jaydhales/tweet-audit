mod config;
mod gemini;
mod routes;
mod state;
mod tweets;

use crate::config::load_config;
use crate::gemini::{
    ChatMessage, ChatMessageType, ChatRequest, ChatText, GEMINI_MODEL, GeminiClient,
};
use crate::routes::{health_check, run_audit};
use crate::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router, routing::get};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    //Load Config
    let config = load_config().unwrap();

    // Load AI Client
    let client = GeminiClient::new();

    let app_state = Arc::new(AppState { client, config });

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/health", get(health_check))
        .route("/test-ai", get(test_ai))
        .route("/audit", post(run_audit))
        .with_state(app_state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🚀 Server running on port 3000");
    axum::serve(listener, app).await.unwrap();

}

#[derive(Deserialize, Serialize)]
pub struct TestResponse {
    msg: String,
}

pub async fn test_ai(
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<TestResponse>), StatusCode> {
    let question = "Why is the sky red?";

    let chat_req = ChatRequest {
        system_instruction: ChatMessage {
            parts: vec![ChatMessageType::Text(ChatText {
                text: "Return a one line response in JSON with format \
           {\
                msg: String\
            }\
            "
                .into(),
            })],
        },
        contents: vec![ChatMessage {
            parts: vec![ChatMessageType::Text(ChatText {
                text: question.into(),
            })],
        }],
    };

    let json_str = state.client.exec_chat(chat_req).await.map_err(|e| {
        println!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let resp = serde_json::from_str::<TestResponse>(&json_str).map_err(|e| {
        println!("Failed to parse Gemini response: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::OK, Json(resp)))
}

// GET /
// GET /health
// GET /config
// POST /audit
// GET /audit/results
// GET /audit/results.csv
