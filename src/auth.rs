use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use hmac::{Hmac, Mac};
use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};
use rand::Rng;
use sha1::Sha1;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::Config;

/// RFC 3986 unreserved characters: ALPHA, DIGIT, '-', '.', '_', '~'
const ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

pub fn percent_encode(input: &str) -> String {
    utf8_percent_encode(input, ENCODE_SET).to_string()
}

fn generate_nonce() -> String {
    let mut rng = rand::thread_rng();
    (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..36);
            if idx < 10 {
                (b'0' + idx) as char
            } else {
                (b'a' + idx - 10) as char
            }
        })
        .collect()
}

fn generate_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string()
}

/// Flexible OAuth 1.0a header builder that supports the 3-legged flow.
/// - `token`: None for request_token step, Some for subsequent steps
/// - `extra_params`: additional params like oauth_callback or oauth_verifier
pub fn build_flexible_oauth_header(
    consumer_key: &str,
    consumer_secret: &str,
    token: Option<&str>,
    token_secret: &str,
    method: &str,
    url: &str,
    extra_params: &[(&str, &str)],
) -> String {
    let nonce = generate_nonce();
    let timestamp = generate_timestamp();

    let mut params: Vec<(&str, &str)> = vec![
        ("oauth_consumer_key", consumer_key),
        ("oauth_nonce", &nonce),
        ("oauth_signature_method", "HMAC-SHA1"),
        ("oauth_timestamp", &timestamp),
        ("oauth_version", "1.0"),
    ];

    if let Some(t) = token {
        params.push(("oauth_token", t));
    }

    for &(k, v) in extra_params {
        params.push((k, v));
    }

    // Sort parameters lexicographically
    params.sort_by_key(|&(k, _)| k);

    // Build parameter string
    let param_string: String = params
        .iter()
        .map(|(k, v)| format!("{}={}", percent_encode(k), percent_encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    // Build signature base string
    let base_string = format!(
        "{}&{}&{}",
        method.to_uppercase(),
        percent_encode(url),
        percent_encode(&param_string)
    );

    // Build signing key
    let signing_key = format!(
        "{}&{}",
        percent_encode(consumer_secret),
        percent_encode(token_secret)
    );

    // HMAC-SHA1
    let mut mac = Hmac::<Sha1>::new_from_slice(signing_key.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(base_string.as_bytes());
    let signature = STANDARD.encode(mac.finalize().into_bytes());

    // Build Authorization header
    let mut header_params: Vec<(&str, String)> = vec![
        ("oauth_consumer_key", percent_encode(consumer_key)),
        ("oauth_nonce", percent_encode(&nonce)),
        ("oauth_signature", percent_encode(&signature)),
        ("oauth_signature_method", "HMAC-SHA1".to_string()),
        ("oauth_timestamp", percent_encode(&timestamp)),
        ("oauth_version", "1.0".to_string()),
    ];

    if let Some(t) = token {
        header_params.push(("oauth_token", percent_encode(t)));
    }

    for &(k, v) in extra_params {
        header_params.push((k, percent_encode(v)));
    }

    header_params.sort_by_key(|(k, _)| *k);

    let header_str = header_params
        .iter()
        .map(|(k, v)| format!("{k}=\"{v}\""))
        .collect::<Vec<_>>()
        .join(", ");

    format!("OAuth {header_str}")
}

/// Convenience wrapper for authenticated API calls (existing behavior).
pub fn build_oauth_header(config: &Config, method: &str, url: &str) -> String {
    build_flexible_oauth_header(
        &config.api_key,
        &config.api_secret,
        Some(&config.access_token),
        &config.access_token_secret,
        method,
        url,
        &[],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_encode_unreserved_unchanged() {
        assert_eq!(percent_encode("abc123"), "abc123");
        assert_eq!(percent_encode("a-b_c.d~e"), "a-b_c.d~e");
    }

    #[test]
    fn percent_encode_special_chars() {
        assert_eq!(percent_encode("hello world"), "hello%20world");
        assert_eq!(percent_encode("a&b=c"), "a%26b%3Dc");
        assert_eq!(percent_encode("100%"), "100%25");
    }

    #[test]
    fn flexible_header_starts_with_oauth() {
        let header = build_flexible_oauth_header(
            "consumer_key",
            "consumer_secret",
            Some("token"),
            "token_secret",
            "GET",
            "https://api.x.com/2/tweets",
            &[],
        );
        assert!(header.starts_with("OAuth "));
    }

    #[test]
    fn flexible_header_contains_required_params() {
        let header = build_flexible_oauth_header(
            "my_key",
            "my_secret",
            Some("my_token"),
            "my_token_secret",
            "POST",
            "https://api.x.com/2/tweets",
            &[],
        );
        assert!(header.contains("oauth_consumer_key=\"my_key\""));
        assert!(header.contains("oauth_token=\"my_token\""));
        assert!(header.contains("oauth_signature_method=\"HMAC-SHA1\""));
        assert!(header.contains("oauth_version=\"1.0\""));
        assert!(header.contains("oauth_signature="));
        assert!(header.contains("oauth_nonce="));
        assert!(header.contains("oauth_timestamp="));
    }

    #[test]
    fn flexible_header_without_token() {
        let header = build_flexible_oauth_header(
            "my_key",
            "my_secret",
            None,
            "",
            "POST",
            "https://api.x.com/oauth/request_token",
            &[("oauth_callback", "http://localhost:8080/callback")],
        );
        assert!(!header.contains("oauth_token="));
        assert!(header.contains("oauth_callback="));
    }

    #[test]
    fn flexible_header_with_extra_params() {
        let header = build_flexible_oauth_header(
            "key",
            "secret",
            Some("tok"),
            "tok_secret",
            "POST",
            "https://api.x.com/oauth/access_token",
            &[("oauth_verifier", "verifier123")],
        );
        assert!(header.contains("oauth_verifier=\"verifier123\""));
    }

    #[test]
    fn build_oauth_header_wraps_flexible() {
        let config = Config {
            api_key: "ck".to_string(),
            api_secret: "cs".to_string(),
            access_token: "at".to_string(),
            access_token_secret: "ats".to_string(),
        };
        let header = build_oauth_header(&config, "GET", "https://api.x.com/2/tweets");
        assert!(header.starts_with("OAuth "));
        assert!(header.contains("oauth_consumer_key=\"ck\""));
        assert!(header.contains("oauth_token=\"at\""));
    }
}
