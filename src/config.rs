use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

pub struct Config {
    pub api_key: String,
    pub api_secret: String,
    pub access_token: String,
    pub access_token_secret: String,
}

#[derive(Serialize, Deserialize)]
pub struct Credentials {
    pub access_token: String,
    pub access_token_secret: String,
    pub screen_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct ApiKeys {
    pub api_key: String,
    pub api_secret: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token_secret: Option<String>,
}

fn config_dir() -> PathBuf {
    dirs::config_dir()
        .expect("Could not determine config directory")
        .join("xcli")
}

pub fn credentials_path() -> PathBuf {
    config_dir().join("credentials.json")
}

pub fn keys_path() -> PathBuf {
    config_dir().join("keys.json")
}

impl Credentials {
    pub fn load() -> Option<Self> {
        Self::load_from(&credentials_path())
    }

    pub fn save(&self) -> Result<(), String> {
        self.save_to(&credentials_path())
    }

    pub fn delete() -> Result<(), String> {
        Self::delete_at(&credentials_path())
    }

    pub fn load_from(path: &PathBuf) -> Option<Self> {
        let data = fs::read_to_string(path).ok()?;
        serde_json::from_str(&data).ok()
    }

    pub fn save_to(&self, path: &PathBuf) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {e}"))?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize credentials: {e}"))?;
        fs::write(path, json)
            .map_err(|e| format!("Failed to write credentials: {e}"))?;
        Ok(())
    }

    pub fn delete_at(path: &PathBuf) -> Result<(), String> {
        if path.exists() {
            fs::remove_file(path)
                .map_err(|e| format!("Failed to delete credentials: {e}"))?;
        }
        Ok(())
    }
}

impl ApiKeys {
    pub fn load() -> Option<Self> {
        Self::load_from(&keys_path())
    }

    pub fn save(&self) -> Result<(), String> {
        self.save_to(&keys_path())
    }

    pub fn load_from(path: &PathBuf) -> Option<Self> {
        let data = fs::read_to_string(path).ok()?;
        serde_json::from_str(&data).ok()
    }

    pub fn save_to(&self, path: &PathBuf) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {e}"))?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize keys: {e}"))?;
        fs::write(path, json).map_err(|e| format!("Failed to write keys: {e}"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    fn test_creds() -> Credentials {
        Credentials {
            access_token: "token123".to_string(),
            access_token_secret: "secret456".to_string(),
            screen_name: "testuser".to_string(),
        }
    }

    fn temp_path(name: &str) -> PathBuf {
        temp_dir().join(format!("xcli_test_{}_{name}.json", std::process::id()))
    }

    #[test]
    fn save_and_load() {
        let path = temp_path("save_load");
        let creds = test_creds();
        creds.save_to(&path).unwrap();

        let loaded = Credentials::load_from(&path).unwrap();
        assert_eq!(loaded.access_token, "token123");
        assert_eq!(loaded.access_token_secret, "secret456");
        assert_eq!(loaded.screen_name, "testuser");

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn delete_removes_file() {
        let path = temp_path("delete");
        let creds = test_creds();
        creds.save_to(&path).unwrap();
        assert!(path.exists());

        Credentials::delete_at(&path).unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn delete_nonexistent_is_ok() {
        let path = temp_dir().join("xcli_nonexistent_999.json");
        assert!(Credentials::delete_at(&path).is_ok());
    }

    #[test]
    fn load_missing_returns_none() {
        let path = temp_dir().join("xcli_missing_999.json");
        assert!(Credentials::load_from(&path).is_none());
    }

    #[test]
    fn load_invalid_json_returns_none() {
        let path = temp_path("invalid_json");
        fs::write(&path, "not json").unwrap();
        assert!(Credentials::load_from(&path).is_none());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn api_keys_save_and_load() {
        let path = temp_path("api_keys");
        let keys = ApiKeys {
            api_key: "key1".to_string(),
            api_secret: "secret1".to_string(),
            access_token: Some("at".to_string()),
            access_token_secret: Some("ats".to_string()),
        };
        keys.save_to(&path).unwrap();

        let loaded = ApiKeys::load_from(&path).unwrap();
        assert_eq!(loaded.api_key, "key1");
        assert_eq!(loaded.api_secret, "secret1");
        assert_eq!(loaded.access_token.unwrap(), "at");
        assert_eq!(loaded.access_token_secret.unwrap(), "ats");

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn api_keys_optional_tokens() {
        let path = temp_path("api_keys_no_tokens");
        let keys = ApiKeys {
            api_key: "key2".to_string(),
            api_secret: "secret2".to_string(),
            access_token: None,
            access_token_secret: None,
        };
        keys.save_to(&path).unwrap();

        let loaded = ApiKeys::load_from(&path).unwrap();
        assert_eq!(loaded.api_key, "key2");
        assert!(loaded.access_token.is_none());
        assert!(loaded.access_token_secret.is_none());

        // Verify optional fields are omitted from JSON
        let json = fs::read_to_string(&path).unwrap();
        assert!(!json.contains("access_token"));

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn api_keys_load_missing_returns_none() {
        let path = temp_dir().join("xcli_keys_missing_999.json");
        assert!(ApiKeys::load_from(&path).is_none());
    }
}

impl Config {
    /// Load config with priority: credentials.json → keys.json → .env
    pub fn load() -> Result<Self, String> {
        dotenvy::dotenv().ok();

        let keys = ApiKeys::load();

        let api_key = env::var("X_API_KEY")
            .ok()
            .or_else(|| keys.as_ref().map(|k| k.api_key.clone()))
            .ok_or("X_API_KEY not set. Run `xcli auth setup` or set it in .env")?;
        let api_secret = env::var("X_API_SECRET")
            .ok()
            .or_else(|| keys.as_ref().map(|k| k.api_secret.clone()))
            .ok_or("X_API_SECRET not set. Run `xcli auth setup` or set it in .env")?;

        // 1) credentials.json (OAuth tokens)
        if let Some(creds) = Credentials::load() {
            return Ok(Config {
                api_key,
                api_secret,
                access_token: creds.access_token,
                access_token_secret: creds.access_token_secret,
            });
        }

        // 2) keys.json access tokens
        if let Some(ref k) = keys {
            if let (Some(at), Some(ats)) = (&k.access_token, &k.access_token_secret) {
                return Ok(Config {
                    api_key,
                    api_secret,
                    access_token: at.clone(),
                    access_token_secret: ats.clone(),
                });
            }
        }

        // 3) .env access tokens
        let access_token = env::var("X_ACCESS_TOKEN")
            .map_err(|_| "Not logged in. Run `xcli auth login` or set X_ACCESS_TOKEN in .env")?;
        let access_token_secret = env::var("X_ACCESS_TOKEN_SECRET")
            .map_err(|_| "Not logged in. Run `xcli auth login` or set X_ACCESS_TOKEN_SECRET in .env")?;

        Ok(Config {
            api_key,
            api_secret,
            access_token,
            access_token_secret,
        })
    }

    /// Load only api_key and api_secret (for OAuth flow before user tokens exist).
    /// Priority: keys.json → .env
    pub fn load_consumer_only() -> Result<(String, String), String> {
        dotenvy::dotenv().ok();

        if let Some(keys) = ApiKeys::load() {
            return Ok((keys.api_key, keys.api_secret));
        }

        let api_key = env::var("X_API_KEY")
            .map_err(|_| "X_API_KEY not set. Run `xcli auth setup` or set it in .env")?;
        let api_secret = env::var("X_API_SECRET")
            .map_err(|_| "X_API_SECRET not set. Run `xcli auth setup` or set it in .env")?;

        Ok((api_key, api_secret))
    }
}
