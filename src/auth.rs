use hmac::{Hmac, Mac};
use sha1::Sha1;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::Config;

/// RFC 3986 unreserved characters: ALPHA, DIGIT, '-', '.', '_', '~'
const ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

fn percent_encode(input: &str) -> String {
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

pub fn build_oauth_header(
    config: &Config,
    method: &str,
    url: &str,
) -> String {
    let nonce = generate_nonce();
    let timestamp = generate_timestamp();

    let mut params = vec![
        ("oauth_consumer_key", config.api_key.as_str()),
        ("oauth_nonce", &nonce),
        ("oauth_signature_method", "HMAC-SHA1"),
        ("oauth_timestamp", &timestamp),
        ("oauth_token", config.access_token.as_str()),
        ("oauth_version", "1.0"),
    ];

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
        percent_encode(&config.api_secret),
        percent_encode(&config.access_token_secret)
    );

    // HMAC-SHA1
    let mut mac = Hmac::<Sha1>::new_from_slice(signing_key.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(base_string.as_bytes());
    let signature = STANDARD.encode(mac.finalize().into_bytes());

    // Build Authorization header
    format!(
        "OAuth oauth_consumer_key=\"{}\", oauth_nonce=\"{}\", oauth_signature=\"{}\", oauth_signature_method=\"HMAC-SHA1\", oauth_timestamp=\"{}\", oauth_token=\"{}\", oauth_version=\"1.0\"",
        percent_encode(&config.api_key),
        percent_encode(&nonce),
        percent_encode(&signature),
        percent_encode(&timestamp),
        percent_encode(&config.access_token),
    )
}
