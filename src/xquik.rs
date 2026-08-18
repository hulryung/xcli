use serde::{Deserialize, Serialize};
use std::env;

const DEFAULT_BASE_URL: &str = "https://xquik.com";

#[derive(Debug, PartialEq)]
pub enum XquikPostResult {
    Posted(String),
    Accepted(String),
}

#[derive(Debug)]
pub struct XquikConfig {
    api_key: String,
    account: String,
    base_url: String,
}

#[derive(Serialize)]
struct CreateTweetRequest<'a> {
    account: &'a str,
    text: &'a str,
}

#[derive(Deserialize)]
struct CreateTweetResponse {
    #[serde(rename = "tweetId")]
    tweet_id: Option<String>,
    #[serde(rename = "writeActionId")]
    write_action_id: Option<String>,
    message: Option<String>,
    error: Option<String>,
}

pub fn backend_enabled() -> bool {
    backend_is_xquik(env::var("XCLI_BACKEND").ok().as_deref())
}

fn backend_is_xquik(value: Option<&str>) -> bool {
    value
        .map(|backend| backend.eq_ignore_ascii_case("xquik"))
        .unwrap_or(false)
}

pub fn config_from_env() -> Result<XquikConfig, String> {
    config_from_values(
        env::var("XQUIK_API_KEY").ok().as_deref(),
        env::var("XQUIK_ACCOUNT").ok().as_deref(),
        env::var("XQUIK_API_BASE_URL").ok().as_deref(),
    )
}

fn config_from_values(
    api_key: Option<&str>,
    account: Option<&str>,
    base_url: Option<&str>,
) -> Result<XquikConfig, String> {
    let api_key = api_key.unwrap_or_default().trim();
    if api_key.is_empty() {
        return Err("XQUIK_API_KEY is required when XCLI_BACKEND=xquik".to_string());
    }

    let account = account.unwrap_or_default().trim();
    if account.is_empty() {
        return Err("XQUIK_ACCOUNT is required when XCLI_BACKEND=xquik".to_string());
    }

    let base_url = base_url
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_BASE_URL)
        .trim_end_matches('/')
        .to_string();

    Ok(XquikConfig {
        api_key: api_key.to_string(),
        account: account.to_string(),
        base_url,
    })
}

pub async fn create_tweet(config: &XquikConfig, text: &str) -> Result<XquikPostResult, String> {
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/v1/x/tweets", config.base_url))
        .header("X-API-Key", &config.api_key)
        .json(&CreateTweetRequest {
            account: &config.account,
            text,
        })
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    let status = response.status().as_u16();
    let body = response.text().await.unwrap_or_default();
    parse_response(status, &body)
}

fn parse_response(status: u16, body: &str) -> Result<XquikPostResult, String> {
    let parsed: CreateTweetResponse =
        serde_json::from_str(body).map_err(|e| format!("Failed to parse Xquik response: {e}"))?;

    match status {
        200 => parsed
            .tweet_id
            .map(XquikPostResult::Posted)
            .ok_or_else(|| "Xquik response did not include tweetId".to_string()),
        202 => parsed
            .write_action_id
            .map(XquikPostResult::Accepted)
            .ok_or_else(|| "Xquik response did not include writeActionId".to_string()),
        _ => Err(parsed
            .message
            .or(parsed.error)
            .unwrap_or_else(|| format!("Xquik request failed with status {status}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_flag_accepts_xquik() {
        assert!(backend_is_xquik(Some("xquik")));
        assert!(backend_is_xquik(Some("XQUIK")));
        assert!(!backend_is_xquik(Some("api")));
        assert!(!backend_is_xquik(None));
    }

    #[test]
    fn config_requires_api_key() {
        let err = config_from_values(None, Some("@xquik"), None).unwrap_err();
        assert!(err.contains("XQUIK_API_KEY"));
    }

    #[test]
    fn config_defaults_base_url() {
        let config = config_from_values(Some("key"), Some("@xquik"), None).unwrap();
        assert_eq!(config.base_url, DEFAULT_BASE_URL);
    }

    #[test]
    fn config_trims_custom_base_url() {
        let config =
            config_from_values(Some("key"), Some("@xquik"), Some("https://example.test/")).unwrap();
        assert_eq!(config.base_url, "https://example.test");
    }

    #[test]
    fn parse_posted_response() {
        let result = parse_response(200, r#"{"tweetId":"123"}"#).unwrap();
        assert_eq!(result, XquikPostResult::Posted("123".to_string()));
    }

    #[test]
    fn parse_accepted_response() {
        let result = parse_response(202, r#"{"writeActionId":"456"}"#).unwrap();
        assert_eq!(result, XquikPostResult::Accepted("456".to_string()));
    }

    #[test]
    fn parse_error_message() {
        let err = parse_response(401, r#"{"message":"Unauthorized"}"#).unwrap_err();
        assert_eq!(err, "Unauthorized");
    }
}
