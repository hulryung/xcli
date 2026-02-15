use serde::{Deserialize, Serialize};

use crate::auth::build_oauth_header;
use crate::config::Config;

const TWEETS_URL: &str = "https://api.x.com/2/tweets";

#[derive(Serialize)]
struct CreateTweetBody {
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reply: Option<ReplyTo>,
}

#[derive(Serialize)]
struct ReplyTo {
    in_reply_to_tweet_id: String,
}

#[derive(Deserialize)]
struct CreateTweetResponse {
    data: TweetData,
}

#[derive(Deserialize)]
struct TweetData {
    id: String,
}

#[derive(Deserialize)]
struct DeleteTweetResponse {
    data: DeleteData,
}

#[derive(Deserialize)]
struct DeleteData {
    deleted: bool,
}

pub async fn create_tweet(
    config: &Config,
    text: &str,
    reply_to: Option<&str>,
) -> Result<String, String> {
    let auth_header = build_oauth_header(config, "POST", TWEETS_URL);

    let client = reqwest::Client::new();
    let body = CreateTweetBody {
        text: text.to_string(),
        reply: reply_to.map(|id| ReplyTo {
            in_reply_to_tweet_id: id.to_string(),
        }),
    };

    let resp = client
        .post(TWEETS_URL)
        .header("Authorization", &auth_header)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("API error ({status}): {body}"));
    }

    let data: CreateTweetResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {e}"))?;

    Ok(data.data.id)
}

pub async fn delete_tweet(config: &Config, id: &str) -> Result<bool, String> {
    let url = format!("{TWEETS_URL}/{id}");
    let auth_header = build_oauth_header(config, "DELETE", &url);

    let client = reqwest::Client::new();

    let resp = client
        .delete(&url)
        .header("Authorization", &auth_header)
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("API error ({status}): {body}"));
    }

    let data: DeleteTweetResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {e}"))?;

    Ok(data.data.deleted)
}

pub struct ThreadError {
    pub posted_ids: Vec<String>,
    pub failed_index: usize,
    pub error: String,
}

pub async fn create_thread(
    config: &Config,
    chunks: &[String],
) -> Result<Vec<String>, ThreadError> {
    let mut posted_ids: Vec<String> = Vec::new();

    for (i, chunk) in chunks.iter().enumerate() {
        let reply_to = posted_ids.last().map(|s| s.as_str());
        match create_tweet(config, chunk, reply_to).await {
            Ok(id) => posted_ids.push(id),
            Err(e) => {
                return Err(ThreadError {
                    posted_ids,
                    failed_index: i,
                    error: e,
                });
            }
        }
    }

    Ok(posted_ids)
}
