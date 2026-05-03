use genai::resolver::{AuthData, AuthResolver};
use genai::{Client as GenAIClient, ModelIden};
use reqwest::header::HeaderMap;
use serde::Serialize;

pub const GEMINI_MODEL: &str = "gemini-2.5-flash";
const GEMINI_ENV_KEY: &str = "GEMINI_API_KEY";

use serde::Deserialize;

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

impl ChatResponse {
    /// Returns raw text at candidates[0].content.parts[0].text (if present)
    pub fn first_text(&self) -> Option<&str> {
        self.candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.as_str())
    }

    /// Returns text with optional markdown code fences removed.
    /// Handles:
    /// - ```json ... ```
    /// - ``` ... ```
    pub fn first_text_stripped(&self) -> Option<String> {
        self.first_text().map(strip_code_fence)
    }
}

/// Utility for stripping markdown code fences from model output.
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
    pub fn new() -> Self {
        if std::env::var(GEMINI_ENV_KEY).is_err() {
            panic!("env var not set: {}", GEMINI_ENV_KEY);
        }

        GeminiClient {
            client: reqwest::Client::new(),
            base_url: format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
                GEMINI_MODEL
            ),
        }
    }

    pub async fn exec_chat(
        &self,
        request: ChatRequest,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut headers = HeaderMap::new();
        headers.insert("x-goog-api-key", std::env::var(GEMINI_ENV_KEY)?.parse()?);
        let raw_res = self
            .client
            .post(&self.base_url)
            .headers(headers)
            .json(&request)
            .send()
            .await?
            .text()
            .await?;

        println!("Server Response: \n{}", raw_res);

        let res = serde_json::from_str::<ChatResponse>(&raw_res)?;

        // let stripped_res = res.first_text_stripped().unwrap();

        if let Some(stripped) = res.first_text_stripped() {
            Ok(stripped)
        } else {
            Err("Response Extraction failed".into())
        }
    }
}

// pub async fn load_gemini_client() -> Result<GenAIClient, Box<dyn std::error::Error>> {
//     // Validate env
//     if std::env::var(GEMINI_ENV_KEY).is_err() {
//         return Err(format!("env var not set: {}", GEMINI_ENV_KEY).into());
//     }
//
//     let auth_resolver = AuthResolver::from_resolver_fn(
//         |model_iden: ModelIden| -> Result<Option<AuthData>, genai::resolver::Error> {
//             let ModelIden {
//                 adapter_kind,
//                 model_name,
//             } = model_iden;
//
//             let key = std::env::var(GEMINI_ENV_KEY).map_err(|_| {
//                 genai::resolver::Error::ApiKeyEnvNotFound {
//                     env_name: GEMINI_ENV_KEY.to_string(),
//                 }
//             })?;
//
//             Ok(Some(AuthData::from_single(key)))
//         },
//     );
//
//     let client = GenAIClient::builder()
//         .with_auth_resolver(auth_resolver)
//         .build();
//
//     client.resolve_service_target(GEMINI_MODEL).await?;
//
//     Ok(client)
// }
