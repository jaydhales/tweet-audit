// POST /audit
// GET /audit/results
// GET /audit/results.csv

use crate::gemini::{ChatData, ChatMessageType, ChatRequest, ChatText, GEMINI_MODEL, InlineData};
use crate::state::AppState;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize, Serialize)]
pub struct AuditedTweets {
    tweet_url: String,
    criteria: String,
    deleted: bool,
}

pub async fn run_audit(
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<AuditedTweets>), StatusCode> {
    let chat_req = ChatRequest {
        system_instruction: crate::gemini::ChatMessage {
            parts: vec![ChatMessageType::Text(ChatText {
                text: "From the uploaded json archive, \
                evaluate all tweets based on the following criteria: unprofessional \
                and return them in a json with format: \
                {\
                    tweet_url: String,\
                    criteria: String,\
                    deleted: bool\
                }\
                "
                .into(),
            })],
        },
        contents: vec![crate::gemini::ChatMessage {
            parts: vec![ChatMessageType::Data(ChatData {
                inline_data: InlineData {
                    mime_type: "application/json".into(),
                    data: state.config.archive.clone(),
                },
            })],
        }],
    };

    println!("Chat Request \n:{:?}", chat_req);

    println!("===== Uploading File =====");
    let json_str = state.client.exec_chat(chat_req).await.map_err(|e| {
        println!("Chat Req Error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let resp = serde_json::from_str::<AuditedTweets>(&json_str).map_err(|e| {
        println!("Failed to parse Gemini response: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::OK, Json(resp)))
}
