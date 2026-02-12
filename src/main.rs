mod api;
mod auth;
mod config;
mod oauth;

use clap::{Parser, Subcommand};
use config::{Config, Credentials};

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
                    eprintln!("Set X_API_KEY and X_API_SECRET in your .env file.");
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
    }
}
