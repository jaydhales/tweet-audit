use base64::{Engine as _, engine::general_purpose};
use std::fs;
use std::path::PathBuf;

#[derive(serde::Deserialize)]
struct RawConfig {
    archive_path: PathBuf,
    output_csv: PathBuf,
}

pub struct Config {
    pub archive: String,
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

    let archive_raw = fs::read(&config.archive_path)?;
    println!("File Size: {}kb", archive_raw.len() / 1024);

    let archive = general_purpose::STANDARD.encode(&archive_raw);

    if archive.is_empty() {
        return Err("Invalid Zip File".into());
    }

    let output: Vec<FlaggedTweet> = csv::Reader::from_path(&config.output_csv)?
        .deserialize()
        .map(|f| f.unwrap())
        .collect();

    Ok(Config { archive, output })
}
