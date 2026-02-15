use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;

use crate::auth::build_flexible_oauth_header;
use crate::config::Credentials;

const REQUEST_TOKEN_URL: &str = "https://api.x.com/oauth/request_token";
const AUTHORIZE_URL: &str = "https://api.x.com/oauth/authorize";
const ACCESS_TOKEN_URL: &str = "https://api.x.com/oauth/access_token";
const CALLBACK_PORT: u16 = 18923;
const CALLBACK_URL: &str = "http://127.0.0.1:18923/callback";

pub fn parse_form_body(body: &str) -> HashMap<String, String> {
    body.split('&')
        .filter_map(|pair| {
            let mut it = pair.splitn(2, '=');
            Some((it.next()?.to_string(), it.next()?.to_string()))
        })
        .collect()
}

pub async fn start_login(api_key: &str, api_secret: &str) -> Result<Credentials, String> {
    // 1. Bind to fixed port
    let listener = TcpListener::bind(format!("127.0.0.1:{CALLBACK_PORT}"))
        .map_err(|e| format!("Failed to bind local server on port {CALLBACK_PORT}: {e}"))?;
    let callback_url = CALLBACK_URL;

    // 2. Get request token
    let auth_header = build_flexible_oauth_header(
        api_key,
        api_secret,
        None,
        "", // no token secret yet
        "POST",
        REQUEST_TOKEN_URL,
        &[("oauth_callback", callback_url)],
    );

    let client = reqwest::Client::new();
    let resp = client
        .post(REQUEST_TOKEN_URL)
        .header("Authorization", &auth_header)
        .send()
        .await
        .map_err(|e| format!("Request token request failed: {e}"))?;

    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(format!("Request token failed ({status}): {body}"));
    }

    let params = parse_form_body(&body);
    let request_token = params
        .get("oauth_token")
        .ok_or("Missing oauth_token in response")?
        .clone();
    let request_token_secret = params
        .get("oauth_token_secret")
        .ok_or("Missing oauth_token_secret in response")?
        .clone();

    // 3. Open browser for authorization
    let authorize_url = format!("{AUTHORIZE_URL}?oauth_token={request_token}");
    println!("Opening browser for authorization...");
    println!("If the browser doesn't open, visit: {authorize_url}");
    let _ = open::that(&authorize_url);

    // 4. Wait for callback
    println!("Waiting for authorization callback...");
    let (oauth_token, oauth_verifier) = wait_for_callback(&listener)?;

    if oauth_token != request_token {
        return Err("OAuth token mismatch".to_string());
    }

    // 5. Exchange for access token
    let auth_header = build_flexible_oauth_header(
        api_key,
        api_secret,
        Some(&request_token),
        &request_token_secret,
        "POST",
        ACCESS_TOKEN_URL,
        &[("oauth_verifier", &oauth_verifier)],
    );

    let resp = client
        .post(ACCESS_TOKEN_URL)
        .header("Authorization", &auth_header)
        .send()
        .await
        .map_err(|e| format!("Access token request failed: {e}"))?;

    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(format!("Access token failed ({status}): {body}"));
    }

    let params = parse_form_body(&body);
    let access_token = params
        .get("oauth_token")
        .ok_or("Missing oauth_token in access response")?
        .clone();
    let access_token_secret = params
        .get("oauth_token_secret")
        .ok_or("Missing oauth_token_secret in access response")?
        .clone();
    let screen_name = params
        .get("screen_name")
        .ok_or("Missing screen_name in access response")?
        .clone();

    Ok(Credentials {
        access_token,
        access_token_secret,
        screen_name,
    })
}

pub fn wait_for_callback(listener: &TcpListener) -> Result<(String, String), String> {
    let (mut stream, _) = listener
        .accept()
        .map_err(|e| format!("Failed to accept connection: {e}"))?;

    let mut buf = [0u8; 4096];
    let n = stream
        .read(&mut buf)
        .map_err(|e| format!("Failed to read request: {e}"))?;
    let request = String::from_utf8_lossy(&buf[..n]);

    // Parse GET /callback?oauth_token=...&oauth_verifier=... HTTP/1.1
    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .ok_or("Invalid HTTP request")?;

    let query = path
        .split('?')
        .nth(1)
        .ok_or("No query string in callback")?;

    let params = parse_form_body(query);
    let oauth_token = params
        .get("oauth_token")
        .ok_or("Missing oauth_token in callback")?
        .clone();
    let oauth_verifier = params
        .get("oauth_verifier")
        .ok_or("Missing oauth_verifier in callback")?
        .clone();

    // Respond with success page
    let html = r#"<!DOCTYPE html>
<html><body style="font-family:system-ui;text-align:center;padding:60px">
<h1>Authorized!</h1>
<p>You can close this tab and return to the terminal.</p>
</body></html>"#;

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        html.len(),
        html
    );
    let _ = stream.write_all(response.as_bytes());

    Ok((oauth_token, oauth_verifier))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as IoWrite;
    use std::net::TcpStream;

    #[test]
    fn parse_form_body_basic() {
        let result = parse_form_body("oauth_token=abc&oauth_token_secret=def&confirmed=true");
        assert_eq!(result.get("oauth_token").unwrap(), "abc");
        assert_eq!(result.get("oauth_token_secret").unwrap(), "def");
        assert_eq!(result.get("confirmed").unwrap(), "true");
    }

    #[test]
    fn parse_form_body_empty() {
        let result = parse_form_body("");
        assert!(result.is_empty());
    }

    #[test]
    fn parse_form_body_single_pair() {
        let result = parse_form_body("key=value");
        assert_eq!(result.len(), 1);
        assert_eq!(result.get("key").unwrap(), "value");
    }

    #[test]
    fn parse_form_body_value_with_equals() {
        let result = parse_form_body("key=val=ue");
        assert_eq!(result.get("key").unwrap(), "val=ue");
    }

    #[test]
    fn wait_for_callback_parses_request() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        let handle = std::thread::spawn(move || wait_for_callback(&listener));

        // Simulate browser callback
        let mut stream = TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        stream
            .write_all(b"GET /callback?oauth_token=tok123&oauth_verifier=ver456 HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .unwrap();

        let (token, verifier) = handle.join().unwrap().unwrap();
        assert_eq!(token, "tok123");
        assert_eq!(verifier, "ver456");
    }

    #[test]
    fn wait_for_callback_missing_verifier() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        let handle = std::thread::spawn(move || wait_for_callback(&listener));

        let mut stream = TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        stream
            .write_all(b"GET /callback?oauth_token=tok123 HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .unwrap();

        let result = handle.join().unwrap();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("oauth_verifier"));
    }
}
