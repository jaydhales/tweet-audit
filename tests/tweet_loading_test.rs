use tweet_audit::tweets::load_tweets;
use std::path::PathBuf;

#[test]
fn test_load_all_tweets() {
    let path = PathBuf::from("data/tweet/tweets.json");

    // This should not panic
    let result = load_tweets(path);

    assert!(result.is_ok(), "Failed to load tweets: {:?}", result.err());

    let tweets = result.unwrap();

    // Verify we loaded the expected number of tweets
    assert_eq!(tweets.len(), 1301, "Expected 1301 tweets, got {}", tweets.len());

    println!("✅ Successfully loaded {} tweets", tweets.len());
}

#[test]
fn test_initial_variant_tweets() {
    let path = PathBuf::from("data/tweet/tweets.json");
    let tweets = load_tweets(path).expect("Failed to load tweets");

    // Count tweets with 'initial' variant
    let initial_count = tweets.iter()
        .filter(|t| t.tweet.edit_info.initial.is_some())
        .count();

    println!("📊 Tweets with 'initial' variant: {}", initial_count);
    assert!(initial_count > 0, "Should have at least some tweets with 'initial' variant");
}

#[test]
fn test_edit_variant_tweets() {
    let path = PathBuf::from("data/tweet/tweets.json");
    let tweets = load_tweets(path).expect("Failed to load tweets");

    // Count tweets with 'edit' variant
    let edit_count = tweets.iter()
        .filter(|t| t.tweet.edit_info.edit.is_some())
        .count();

    println!("📊 Tweets with 'edit' variant: {}", edit_count);
    // At least 2 tweets should have the 'edit' variant based on the error location
    assert!(edit_count > 0, "Should have at least some tweets with 'edit' variant");
}

#[test]
fn test_all_tweets_have_edit_info() {
    let path = PathBuf::from("data/tweet/tweets.json");
    let tweets = load_tweets(path).expect("Failed to load tweets");

    // Every tweet should have either 'initial' or 'edit'
    for (idx, envelope) in tweets.iter().enumerate() {
        let has_initial = envelope.tweet.edit_info.initial.is_some();
        let has_edit = envelope.tweet.edit_info.edit.is_some();

        assert!(
            has_initial || has_edit,
            "Tweet at index {} has neither 'initial' nor 'edit' variant. ID: {}",
            idx,
            envelope.tweet.id_str
        );
    }

    println!("✅ All {} tweets have valid edit_info structure", tweets.len());
}

#[test]
fn test_specific_tweet_fields() {
    let path = PathBuf::from("data/tweet/tweets.json");
    let tweets = load_tweets(path).expect("Failed to load tweets");

    // Test the first tweet
    let first = &tweets[0];
    assert!(!first.tweet.id_str.is_empty(), "Tweet should have an ID");
    assert!(!first.tweet.full_text.is_empty(), "Tweet should have text");
    assert!(!first.tweet.created_at.is_empty(), "Tweet should have creation date");

    println!("✅ First tweet structure validated");
    println!("   ID: {}", first.tweet.id_str);
    println!("   Text: {}", first.tweet.full_text);
}


