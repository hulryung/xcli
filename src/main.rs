mod api;
mod auth;
mod config;
mod oauth;

use clap::{Parser, Subcommand};
use config::{ApiKeys, Config, Credentials};
use std::io::{self, Write};

#[derive(Parser)]
#[command(name = "xcli", about = "X (Twitter) API CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Post a new tweet
    Tweet {
        /// Text content of the tweet
        text: String,
    },
    /// Delete a tweet by ID
    Delete {
        /// Tweet ID to delete
        id: String,
    },
    /// Manage authentication
    Auth {
        #[command(subcommand)]
        action: AuthAction,
    },
}

#[derive(Subcommand)]
enum AuthAction {
    /// Login via OAuth (opens browser)
    Login,
    /// Logout (delete stored credentials)
    Logout,
    /// Show current auth status
    Status,
    /// Set up API keys
    Setup {
        /// API Key (Consumer Key)
        #[arg(long)]
        api_key: Option<String>,
        /// API Secret (Consumer Secret)
        #[arg(long)]
        api_secret: Option<String>,
        /// Access Token (optional)
        #[arg(long)]
        access_token: Option<String>,
        /// Access Token Secret (optional)
        #[arg(long)]
        access_token_secret: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Auth { action } => handle_auth(action).await,
        Commands::Tweet { text } => {
            let config = load_config_or_exit();
            match api::create_tweet(&config, &text).await {
                Ok(id) => println!("Tweet posted! ID: {id}"),
                Err(e) => {
                    eprintln!("Failed to post tweet: {e}");
                    std::process::exit(1);
                }
            }
        }
        Commands::Delete { id } => {
            let config = load_config_or_exit();
            match api::delete_tweet(&config, &id).await {
                Ok(true) => println!("Tweet {id} deleted."),
                Ok(false) => {
                    eprintln!("Tweet {id} was not deleted.");
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Failed to delete tweet: {e}");
                    std::process::exit(1);
                }
            }
        }
    }
}

fn load_config_or_exit() -> Config {
    match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

async fn handle_auth(action: AuthAction) {
    match action {
        AuthAction::Login => {
            let (api_key, api_secret) = match Config::load_consumer_only() {
                Ok(keys) => keys,
                Err(e) => {
                    eprintln!("Error: {e}");
                    eprintln!("Run `xcli auth setup` or set X_API_KEY and X_API_SECRET in .env.");
                    std::process::exit(1);
                }
            };

            match oauth::start_login(&api_key, &api_secret).await {
                Ok(creds) => {
                    let name = creds.screen_name.clone();
                    if let Err(e) = creds.save() {
                        eprintln!("Failed to save credentials: {e}");
                        std::process::exit(1);
                    }
                    println!("Logged in as @{name}");
                    println!(
                        "Credentials saved to {}",
                        config::credentials_path().display()
                    );
                }
                Err(e) => {
                    eprintln!("Login failed: {e}");
                    std::process::exit(1);
                }
            }
        }
        AuthAction::Logout => {
            if let Err(e) = Credentials::delete() {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
            println!("Logged out. Credentials removed.");
        }
        AuthAction::Status => match Credentials::load() {
            Some(creds) => {
                println!("Logged in as @{}", creds.screen_name);
                println!(
                    "Credentials: {}",
                    config::credentials_path().display()
                );
            }
            None => {
                println!("Not logged in.");
                println!("Run `xcli auth login` to authenticate.");
            }
        },
        AuthAction::Setup {
            api_key,
            api_secret,
            access_token,
            access_token_secret,
        } => {
            let api_key = api_key.unwrap_or_else(|| prompt("API Key"));
            let api_secret = api_secret.unwrap_or_else(|| prompt("API Secret"));
            let access_token = access_token.or_else(|| prompt_optional("Access Token"));
            let access_token_secret =
                access_token_secret.or_else(|| prompt_optional("Access Token Secret"));

            let keys = ApiKeys {
                api_key,
                api_secret,
                access_token,
                access_token_secret,
            };

            if let Err(e) = keys.save() {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
            println!("Keys saved to {}", config::keys_path().display());
        }
    }
}

fn prompt(label: &str) -> String {
    loop {
        print!("{label}: ");
        io::stdout().flush().unwrap();
        let mut buf = String::new();
        io::stdin().read_line(&mut buf).unwrap();
        let val = buf.trim().to_string();
        if !val.is_empty() {
            return val;
        }
        eprintln!("{label} is required.");
    }
}

fn prompt_optional(label: &str) -> Option<String> {
    print!("{label} (optional, press Enter to skip): ");
    io::stdout().flush().unwrap();
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    let val = buf.trim().to_string();
    if val.is_empty() { None } else { Some(val) }
}
