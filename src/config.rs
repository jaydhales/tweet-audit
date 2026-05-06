use crate::AppError;
use crate::tweets::{TweetArchive, load_tweets};
use base64::{Engine as _, engine::general_purpose};
use std::fs;
use std::path::PathBuf;

#[derive(serde::Deserialize)]
struct RawConfig {
    archive_path: PathBuf,
    output_csv: PathBuf,
}

pub struct Config {
    pub archive: TweetArchive,
    pub output: Vec<FlaggedTweet>,
}

#[derive(serde::Deserialize, Debug)]
pub struct FlaggedTweet {
    tweet_url: String,
    deleted: bool,
}

pub fn load_config() -> Result<Config, AppError> {
    let contents = fs::read_to_string("config.yaml")?;
    let config: RawConfig = serde_yaml::from_str(&contents)?;

    tracing::info!("File:  {:?}", &config.archive_path);

    let archive = load_tweets(config.archive_path)?;

    if archive.is_empty() {
        return Err(AppError::Internal("Invalid Json File".into()));
    }
    tracing::info!("✅ Successfully loaded {} tweets!", archive.len());

    tracing::info!(
        "Reading existing flagged tweets from: {:?}",
        &config.output_csv
    );
    let output: Vec<FlaggedTweet> = csv::Reader::from_path(&config.output_csv)?
        .deserialize()
        .map(|f| f.unwrap())
        .collect();
    tracing::info!("✅ Successfully loaded {} flagged tweets!", output.len());

    Ok(Config { archive, output })
}
