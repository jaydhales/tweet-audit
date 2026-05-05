use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Entire file deserializes as Vec<TweetEnvelope>
pub type TweetArchive = Vec<TweetEnvelope>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweetEnvelope {
    pub tweet: Tweet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tweet {
    pub edit_info: EditInfo,
    pub retweeted: bool,
    pub source: String,
    pub entities: Entities,
    pub display_text_range: Vec<String>,
    pub favorite_count: String,
    pub id_str: String,
    pub truncated: bool,
    pub retweet_count: String,
    pub id: String,
    pub created_at: String,
    pub favorited: bool,
    pub full_text: String,
    pub lang: String,

    // Present only on replies
    pub in_reply_to_status_id_str: Option<String>,
    pub in_reply_to_user_id: Option<String>,
    pub in_reply_to_status_id: Option<String>,
    pub in_reply_to_screen_name: Option<String>,
    pub in_reply_to_user_id_str: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditInfo {
    /// For original tweets: { "initial": { ... } }
    #[serde(default)]
    pub initial: Option<EditInitial>,
    /// For edited tweets: { "edit": { "initialTweetId": "...", "editControlInitial": { ... } } }
    #[serde(default)]
    pub edit: Option<EditVariant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditVariant {
    #[serde(rename = "initialTweetId")]
    pub initial_tweet_id: String,
    #[serde(rename = "editControlInitial")]
    pub edit_control_initial: EditInitial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditInitial {
    #[serde(rename = "editTweetIds")]
    pub edit_tweet_ids: Vec<String>,
    #[serde(rename = "editableUntil")]
    pub editable_until: String,
    #[serde(rename = "editsRemaining")]
    pub edits_remaining: String,
    #[serde(rename = "isEditEligible")]
    pub is_edit_eligible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entities {
    // Left flexible because sample shows empty arrays only
    pub hashtags: Vec<serde_json::Value>,
    pub symbols: Vec<serde_json::Value>,
    pub urls: Vec<serde_json::Value>,
    pub user_mentions: Vec<UserMention>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMention {
    pub name: String,
    pub screen_name: String,
    pub indices: Vec<String>,
    pub id_str: String,
    pub id: String,
}

pub fn load_tweets(path: PathBuf) -> Result<TweetArchive, Box<dyn std::error::Error>> {
    println!("📂 Loading tweets from: {:?}", path);
    let raw = fs::read_to_string(path)?;
    let tweets: TweetArchive = serde_json::from_str(&raw)?;
    Ok(tweets)
}