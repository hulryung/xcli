use std::env;

pub struct Config {
    pub api_key: String,
    pub api_secret: String,
    pub access_token: String,
    pub access_token_secret: String,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        dotenvy::dotenv().ok();

        Ok(Config {
            api_key: env::var("X_API_KEY")
                .map_err(|_| "X_API_KEY not set")?,
            api_secret: env::var("X_API_SECRET")
                .map_err(|_| "X_API_SECRET not set")?,
            access_token: env::var("X_ACCESS_TOKEN")
                .map_err(|_| "X_ACCESS_TOKEN not set")?,
            access_token_secret: env::var("X_ACCESS_TOKEN_SECRET")
                .map_err(|_| "X_ACCESS_TOKEN_SECRET not set")?,
        })
    }
}
