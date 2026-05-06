use reqwest::header::HeaderMap;
use serde::Serialize;
use std::time::Duration;
use tokio::time::sleep;

pub const GEMINI_MODEL: &str = "gemini-2.5-flash";
const GEMINI_ENV_KEY: &str = "GEMINI_API_KEY";

use crate::AppError;
use serde::Deserialize;
use tracing::{debug, trace};

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub candidates: Vec<Candidate>,
    #[serde(rename = "usageMetadata")]
    pub usage_metadata: Option<UsageMetadata>,
    #[serde(rename = "modelVersion")]
    pub model_version: Option<String>,
    #[serde(rename = "responseId")]
    pub response_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Candidate {
    pub content: Content,
    #[serde(rename = "finishReason")]
    pub finish_reason: Option<String>,
    pub index: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct Content {
    pub parts: Vec<Part>,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Part {
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct UsageMetadata {
    #[serde(rename = "promptTokenCount")]
    pub prompt_token_count: Option<u32>,
    #[serde(rename = "candidatesTokenCount")]
    pub candidates_token_count: Option<u32>,
    #[serde(rename = "totalTokenCount")]
    pub total_token_count: Option<u32>,
    #[serde(rename = "promptTokensDetails")]
    pub prompt_tokens_details: Option<Vec<TokenDetail>>,
    #[serde(rename = "thoughtsTokenCount")]
    pub thoughts_token_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct TokenDetail {
    pub modality: Option<String>,
    #[serde(rename = "tokenCount")]
    pub token_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct GeminiErrorResponse {
    pub error: GeminiError,
}

#[derive(Debug, Deserialize)]
pub struct GeminiError {
    pub code: u32,
    pub message: String,
    pub status: String,
}

impl ChatResponse {
    /// Returns raw text at candidates[0].content.parts[0].text (if present)
    pub fn first_text(&self) -> Option<&str> {
        self.candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.as_str())
    }

    /// Returns text with optional Markdown code fences removed.
    /// Handles:
    /// - ```json ... ```
    /// - ``` ... ```
    pub fn first_text_stripped(&self) -> Option<String> {
        self.first_text().map(strip_code_fence)
    }
}

/// Utility for stripping Markdown code fences from model output.
fn strip_code_fence(input: &str) -> String {
    input
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim()
        .to_string()
}

pub struct GeminiClient {
    client: reqwest::Client,
    base_url: String,
}

#[derive(Debug, Serialize)]
pub struct ChatText {
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct InlineData {
    pub mime_type: String,
    pub data: String,
}

#[derive(Debug, Serialize)]
pub struct ChatData {
    pub inline_data: InlineData,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ChatMessageType {
    Text(ChatText),
    Data(ChatData),
}

#[derive(Debug, Serialize)]
pub struct ChatMessage {
    pub parts: Vec<ChatMessageType>,
}

#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub system_instruction: ChatMessage,
    pub contents: Vec<ChatMessage>,
}

impl GeminiClient {
    pub fn new() -> Result<Self, AppError> {
        if std::env::var(GEMINI_ENV_KEY).is_err() {
            tracing::error!("env var not set: {}", GEMINI_ENV_KEY);
            return Err(AppError::Internal(format!(
                "env var not set: {}",
                GEMINI_ENV_KEY
            )));
        }

        tracing::info!("🚀 Gemini Client initialized with model: {}", GEMINI_MODEL);
        Ok(Self {
            client: reqwest::Client::new(),
            base_url: format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
                GEMINI_MODEL
            ),
        })
    }

    pub async fn exec_chat(&self, request: ChatRequest) -> Result<String, AppError> {
        self.exec_chat_with_retry(request, 3).await
    }

    pub async fn exec_chat_with_retry(
        &self,
        request: ChatRequest,
        max_retries: u32,
    ) -> Result<String, AppError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-goog-api-key",
            std::env::var(GEMINI_ENV_KEY)
                .map_err(|| AppError::ConfigLoad("Env Error".into()))?
                .parse()
                .map_err(|e| AppError::ConfigLoad(format!("Invalid Header Value: {}", e)))?,
        );

        let mut attempt = 0;
        let mut wait_time = Duration::from_secs(4);

        loop {
            attempt += 1;

            debug!(
                attempt = attempt,
                max_retries = max_retries,
                "Sending chat request"
            );

            let response = self
                .client
                .post(&self.base_url)
                .headers(headers.clone())
                .json(&request)
                .send()
                .await?;

            let status = response.status();
            let raw_res = response.text().await?;

            trace!(status = %status, "Raw response: {}", raw_res);

            // Check for Error
            if status.as_u16() / 100 >= 4 {
                // 400 - 504
                if attempt >= max_retries {
                    return Err(AppError::Internal(
                        format!(
                            "Max retries ({}) exceeded. Last error: {}",
                            max_retries, raw_res
                        )
                        .into(),
                    ));
                }

                // Try to parse error response to get retry delay
                if let Ok(err_resp) = serde_json::from_str::<GeminiErrorResponse>(&raw_res) {
                    tracing::error!("⚠️ Gemini Server Error: {}", err_resp.error.message);

                    // Parse retry delay from error message if present
                    // Message format: "Please retry in 59.402129186s."
                    if let Some(delay_str) = err_resp
                        .error
                        .message
                        .split("Please retry in ")
                        .nth(1)
                        .and_then(|s| s.split("s.").next())
                    {
                        if let Ok(delay_secs) = delay_str.parse::<f64>() {
                            wait_time = Duration::from_secs_f64(delay_secs.ceil() + 1.0);
                        }
                    }
                }

                tracing::info!("⏳ Waiting {:?} before retry...", wait_time);
                sleep(wait_time).await;

                // Exponential backoff for next attempt
                wait_time = wait_time.mul_f32(2.0);
                continue;
            }

            // Check for other HTTP errors
            if !status.is_success() {
                return Err(AppError::Internal(format!("HTTP error {}: {}", status, raw_res).into()));
            }

            // Parse successful response
            let res = serde_json::from_str::<ChatResponse>(&raw_res)?;

            return if let Some(stripped) = res.first_text_stripped() {
                tracing::info!("Stripped Response: {}", stripped);
                Ok(stripped)
            } else {
                Err(AppError::Internal("Response extraction failed".into()))
            };
        }
    }
}