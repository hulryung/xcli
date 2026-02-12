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

pub fn credentials_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .expect("Could not determine config directory")
        .join("xcli");
    config_dir.join("credentials.json")
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
}

impl Config {
    /// Load config: credentials.json first, then env vars as fallback.
    /// api_key and api_secret always come from env.
    pub fn load() -> Result<Self, String> {
        dotenvy::dotenv().ok();

        let api_key = env::var("X_API_KEY")
            .map_err(|_| "X_API_KEY not set")?;
        let api_secret = env::var("X_API_SECRET")
            .map_err(|_| "X_API_SECRET not set")?;

        if let Some(creds) = Credentials::load() {
            return Ok(Config {
                api_key,
                api_secret,
                access_token: creds.access_token,
                access_token_secret: creds.access_token_secret,
            });
        }

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
    pub fn load_consumer_only() -> Result<(String, String), String> {
        dotenvy::dotenv().ok();

        let api_key = env::var("X_API_KEY")
            .map_err(|_| "X_API_KEY not set")?;
        let api_secret = env::var("X_API_SECRET")
            .map_err(|_| "X_API_SECRET not set")?;

        Ok((api_key, api_secret))
    }
}
