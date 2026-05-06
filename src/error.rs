// Even better: Use thiserror
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Failed to load config: {0}")]
    ConfigLoad(String),

    #[error("Failed to load tweets: {0}")]
    TweetLoad(String),

    #[error("Gemini API error: {0}")]
    GeminiApi(String),

    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),
    
    #[error("CSV parse error: {0}")]
    CsvParse(#[from] csv::Error),

    #[error("Yaml parse error: {0}")]
    YamlParse(#[from] serde_yaml::Error),



    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}
