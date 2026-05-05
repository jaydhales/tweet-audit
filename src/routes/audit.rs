// POST /audit
// GET /audit/results
// GET /audit/results.csv

use crate::gemini::{ChatMessage, ChatMessageType, ChatRequest, ChatText};
use crate::state::AppState;
use crate::tweets::TweetEnvelope;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const BATCH_SIZE: usize = 20; // Process 20 tweets per batch

#[derive(Deserialize, Serialize, Debug)]
pub struct AuditedTweets {
    tweet_url: String,
    criteria: String,
    deleted: bool,
}

pub async fn run_audit(
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<Vec<AuditedTweets>>), StatusCode> {

    let total_tweets = state.config.archive.len();
    println!("📊 Starting audit of {} tweets in batches of {}", total_tweets, BATCH_SIZE);

    // Split tweets into batches
    let batches: Vec<&[TweetEnvelope]> = state.config.archive
        .chunks(BATCH_SIZE)
        .collect();

    let total_batches = batches.len();

    println!("📊 Total Batches: {}", total_batches);

    let batched_chat_requests = generate_chat_request(batches);

    let batched_results = process_request(&state, batched_chat_requests).await.unwrap_or_else(|e| {
        eprintln!("❌ Request failed: {}", e);
        Vec::new()
    });

    println!("\n🎉 Audit complete! Total flagged tweets: {}/{}",
        batched_results.len(), total_tweets);

    Ok((StatusCode::OK, Json(batched_results)))
}

fn generate_chat_request(
    requests: Vec<&[TweetEnvelope]>,
) -> ChatRequest {
    ChatRequest {
        system_instruction: ChatMessage {
            parts: vec![ChatMessageType::Text(ChatText {
                text: "From the uploaded json archive, \
                evaluate all tweets based on the following criteria: unprofessional \
                and return only the json format: \
                ```json\
                [{\
                    \"tweet_url\": \"https://x.com/username/status/TWEET_ID\",\
                    \"criteria\": \"reason for flagging\",\
                    \"deleted\": false\
                }]\
                ```\
                IMPORTANT:\
                - The tweet_url MUST be in the format: https://x.com/username/status/TWEET_ID where TWEET_ID is from the tweet's 'id_str' field\
                - If no tweets meet the criteria, return an empty array: []\
                - Return ONLY valid JSON, no other text"
                    .into(),
            })],
        },
        contents: vec![ChatMessage {
            parts: requests.iter().map(|r| ChatMessageType::Text(ChatText {
                text: serde_json::to_string(r).unwrap_or_else(|e| {
                    eprintln!("Failed to serialize batch: {}", e);
                    "[]".to_string()
                })
            })).collect()
        }],
    }
}

async  fn process_request(
    state: &Arc<AppState>,
    chat_req: ChatRequest
) -> Result<Vec<AuditedTweets>, Box<dyn std::error::Error>> {
    let json_str = state.client.exec_chat(chat_req).await?;

    // Handle empty response or non-JSON
    if json_str.trim().is_empty() || json_str.trim() == "[]" {
        return Ok(Vec::new());
    }

    let results = serde_json::from_str::<Vec<AuditedTweets>>(&json_str)
        .map_err(|e| {
            println!("⚠️  Failed to parse  response: {}\nResponse: {}",
                      e, json_str);
            format!("JSON parse error: {}", e)
        })?;

    Ok(results)

}

async fn process_batch(
    state: &Arc<AppState>,
    batch: &[TweetEnvelope],
    batch_num: usize,
) -> Result<Vec<AuditedTweets>, Box<dyn std::error::Error>> {

    let batch_json = serde_json::to_string(&batch)?;

    let chat_req = ChatRequest {
        system_instruction: ChatMessage {
            parts: vec![ChatMessageType::Text(ChatText {
                text: "From the uploaded json archive, \
                evaluate all tweets based on the following criteria: unprofessional \
                and return only the json format: \
                ```json\
                [{\
                    \"tweet_url\": \"https://x.com/username/status/TWEET_ID\",\
                    \"criteria\": \"reason for flagging\",\
                    \"deleted\": false\
                }]\
                ```\
                IMPORTANT:\
                - The tweet_url MUST be in the format: https://x.com/username/status/TWEET_ID where TWEET_ID is from the tweet's 'id_str' field\
                - If no tweets meet the criteria, return an empty array: []\
                - Return ONLY valid JSON, no other text"
                .into(),
            })],
        },
        contents: vec![ChatMessage {
            parts: vec![ChatMessageType::Text(ChatText {
                text: batch_json,
            })],
        }],
    };

    let json_str = state.client.exec_chat(chat_req).await?;

    // Handle empty response or non-JSON
    if json_str.trim().is_empty() || json_str.trim() == "[]" {
        return Ok(Vec::new());
    }

    let results = serde_json::from_str::<Vec<AuditedTweets>>(&json_str)
        .map_err(|e| {
            println!("⚠️  Failed to parse batch {} response: {}\nResponse: {}",
                batch_num, e, json_str);
            format!("JSON parse error: {}", e)
        })?;

    Ok(results)
}
