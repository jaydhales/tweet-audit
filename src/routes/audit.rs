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
use std::time::Duration;
use tokio::time::sleep;

const BATCH_SIZE: usize = 20; // Process 20 tweets per batch
const DELAY_BETWEEN_BATCHES: Duration = Duration::from_secs(3); // 3 second delay

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

    let mut all_results = Vec::new();

    // Split tweets into batches
    let batches: Vec<&[TweetEnvelope]> = state.config.archive
        .chunks(BATCH_SIZE)
        .collect();

    let total_batches = batches.len();

    for (batch_idx, batch) in batches.into_iter().enumerate() {
        let batch_num = batch_idx + 1;
        println!("\n🔄 Processing batch {}/{} ({} tweets)",
            batch_num, total_batches, batch.len());

        match process_batch(&state, batch, batch_num).await {
            Ok(mut results) => {
                println!("✅ Batch {}/{} completed: {} flagged tweets",
                    batch_num, total_batches, results.len());
                all_results.append(&mut results);
            }
            Err(e) => {
                println!("❌ Batch {}/{} failed: {}", batch_num, total_batches, e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }

        // Wait between batches to respect rate limits (except after last batch)
        if batch_num < total_batches {
            println!("⏳ Waiting {:?} before next batch...", DELAY_BETWEEN_BATCHES);
            sleep(DELAY_BETWEEN_BATCHES).await;
        }
    }

    println!("\n🎉 Audit complete! Total flagged tweets: {}/{}",
        all_results.len(), total_tweets);

    Ok((StatusCode::OK, Json(all_results)))
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
