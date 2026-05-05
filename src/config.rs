use base64::{Engine as _, engine::general_purpose};
use std::fs;
use std::path::PathBuf;
use crate::tweets::{load_tweets, TweetArchive};

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

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string("config.yaml")?;
    let config: RawConfig = serde_yaml::from_str(&contents)?;

    println!("File:  {:?}", &config.archive_path);
    
    let archive = load_tweets(config.archive_path)?;

    if archive.is_empty() {
        return Err("Invalid Json File".into());
    }
    println!("✅ Successfully loaded {} tweets!", archive.len());

    println!("Reading existing flagged tweets from: {:?}", &config.output_csv);
    let output: Vec<FlaggedTweet> = csv::Reader::from_path(&config.output_csv)?
        .deserialize()
        .map(|f| f.unwrap())
        .collect();
    println!("✅ Successfully loaded {} flagged tweets!", output.len());

    Ok(Config { archive, output })
}
