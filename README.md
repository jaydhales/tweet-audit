# tweet-audit

Analyse your X (Twitter) archive with Gemini AI and flag tweets that may no longer align with how you want to present yourself.

## Overview

`tweet-audit` is a Rust backend built with Axum. It loads a local X archive, sends tweet data to Gemini for review, and returns flagged tweets through an HTTP API.

The broader goal of the project is to help you review old tweets against custom standards such as:
- unprofessional language
- specific words or topics
- outdated opinions
- any criteria you no longer want associated with your account

## Current Status

This repository is currently an **MVP backend**.

### What works
- Loads tweets from a local archive JSON file
- Handles both standard and edited tweet archive formats
- Starts an HTTP server on port `3000`
- Exposes a health endpoint and audit endpoint
- Sends tweet content to Gemini for evaluation
- Returns flagged tweets as structured JSON
- Includes tests for tweet archive loading

### What is not finished yet
- user-defined audit criteria
- writing results back to CSV
- automated deletion via the X API
- deriving the real username for tweet URLs
- true multi-request batch orchestration across Gemini calls

---

## Intended Output

The original assignment expects a CSV like:

```csv
tweet_url,deleted
https://x.com/username/status/1234567890,false
```

**Current implementation note:** this codebase currently returns **JSON from `POST /audit`** instead of generating the CSV file automatically.

Example response:

```json
[
  {
    "tweet_url": "https://x.com/username/status/1234567890",
    "criteria": "contains unprofessional wording",
    "deleted": false
  }
]
```

---

## Requirements

1. **Rust toolchain**
   - Install from <https://www.rust-lang.org/tools/install>

2. **Gemini API Key**
   - Get one from [Google AI Studio](https://aistudio.google.com/app/apikey)
   - Export it before running the server:

   ```bash
   export GEMINI_API_KEY="your-api-key"
   ```

3. **X archive**
   - Request your archive from X via:
     `Settings and privacy → Your account → Download an archive of your data`

---

## Preparing Your Archive

After downloading and extracting your X archive, place the tweet file at:

```text
data/tweet/tweets.json
```

### Important format note
This implementation expects a **plain JSON file**.

If your archive gives you a file like:

```js
window.YTD.tweets.part0 = [...]
```

you must convert it to raw JSON first, because the loader uses `serde_json::from_str(...)` directly.

---

## Setup

### 1. Configure paths
The app reads `config.yaml` at startup:

```yaml
archive_path: "data/tweet/tweets.json"
output_csv: "data/flagged_tweet.csv"
```

### 2. Create the expected local files
The `data/` directory is gitignored, so you need to create it locally:

```bash
mkdir -p data/tweet
printf "tweet_url,deleted\n" > data/flagged_tweet.csv
```

Then place your archive JSON at:

```text
data/tweet/tweets.json
```

### 3. Run the server

```bash
cargo run
```

The API will start at:

```text
http://localhost:3000
```

---

## API

### `GET /`
Basic root endpoint.

```bash
curl http://localhost:3000/
```

### `GET /health`
Health check endpoint.

```bash
curl http://localhost:3000/health
```

### `POST /audit`
Runs the audit using Gemini.

```bash
curl -X POST http://localhost:3000/audit
```

---

## How the Audit Works

1. The archive is loaded into memory at startup
2. Tweets are grouped in chunks of `20`
3. The server builds a Gemini request from those tweets
4. Gemini is asked to flag tweets that appear **unprofessional**
5. The response is parsed into structured Rust types and returned as JSON

### Current criteria
The audit prompt is currently hard-coded to look for **unprofessional** tweets. Custom criteria are not yet accepted from the API.

---

## Testing

Run the test suite with:

```bash
cargo test
```

Or just the archive-loading tests:

```bash
cargo test --test tweet_loading_test
```

The tests validate:
- archive loading
- both `initial` and `edit` tweet variants
- presence of required fields such as `id_str`, `full_text`, and `created_at`

### Test requirement note
Tests depend on a local archive file at:

```text
data/tweet/tweets.json
```

---

## Project Structure

```text
tweet-audit/
├── README.md
├── Cargo.toml
├── config.yaml
├── routes.http
├── src/
│   ├── config.rs
│   ├── error.rs
│   ├── gemini.rs
│   ├── lib.rs
│   ├── main.rs
│   ├── state.rs
│   ├── telemetry.rs
│   ├── tweets.rs
│   └── routes/
│       ├── audit.rs
│       ├── health_check.rs
│       └── mod.rs
└── tests/
    └── tweet_loading_test.rs
```

---

## Implementation Notes

- **Language/framework:** Rust + Axum
- **AI provider:** Gemini
- **Configuration:** local `config.yaml`
- **Logging:** `tracing`
- **Errors:** shared `AppError` enum via `thiserror`

---

## Known Gaps vs. the Original Assignment

To stay accurate to the current codebase, these are the main differences from the original brief:

1. **JSON response instead of CSV generation**
2. **Hard-coded criteria instead of custom alignment rules**
3. **No deletion workflow yet**
4. **Archive loaded fully at startup**
5. **Startup expects `data/flagged_tweet.csv` to already exist**
6. **Tweet URLs use `username` as a placeholder**

---

## Next Steps

Good next improvements would be:

1. accept custom criteria in `POST /audit`
2. write flagged results to `data/flagged_tweet.csv`
3. process true per-batch Gemini requests with aggregation
4. support raw `tweets.js` archive files automatically
5. derive the real account username for URL generation
6. return structured HTTP error responses

---

## Example Local Workflow

```bash
mkdir -p data/tweet
printf "tweet_url,deleted\n" > data/flagged_tweet.csv
export GEMINI_API_KEY="your-api-key"

# place archive JSON at data/tweet/tweets.json

cargo test
cargo run
curl -X POST http://localhost:3000/audit
```

---

## Summary

Today, `tweet-audit` is best described as:

> a Rust HTTP service that reads an X archive, sends tweets to Gemini, and returns flagged tweets as JSON.

It already covers the core archive-parsing and AI-review flow, and it has a clear path toward the full assignment goal of producing deletion-ready output from custom alignment criteria.
