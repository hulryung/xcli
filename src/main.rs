mod api;
mod auth;
mod config;
mod oauth;
mod thread;

use clap::{Parser, Subcommand};
use config::{ApiKeys, Config, Credentials};
use std::io::{self, Write};

#[derive(Parser)]
#[command(name = "xcli", version, about = "X (Twitter) API CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Post a new tweet (long text is automatically threaded)
    Tweet {
        /// Text content of the tweet
        text: String,
        /// Preview thread split without posting
        #[arg(long)]
        dry_run: bool,
    },
    /// Delete a tweet by ID
    Delete {
        /// Tweet ID to delete
        id: String,
    },
    /// Manage authentication
    #[command(long_about = "Manage authentication\n\nExamples:\n  xcli auth setup --api-key KEY --api-secret SECRET\n  xcli auth login\n  xcli auth status\n  xcli auth logout")]
    Auth {
        #[command(subcommand)]
        action: AuthAction,
    },
}

#[derive(Subcommand)]
enum AuthAction {
    /// Login via OAuth (opens browser)
    #[command(long_about = "Login via OAuth (opens browser)\n\nStarts a 3-legged OAuth flow: opens the browser for authorization,\nthen saves the access token to ~/.config/xcli/credentials.json.\nRequires API keys (run `xcli auth setup` first or set .env).")]
    Login,
    /// Logout (delete stored credentials)
    #[command(long_about = "Logout (delete stored credentials)\n\nRemoves ~/.config/xcli/credentials.json.\nAPI keys in keys.json are kept.")]
    Logout,
    /// Show current auth status
    #[command(long_about = "Show current auth status\n\nDisplays the logged-in screen name and credentials path,\nor indicates that no user is logged in.")]
    Status,
    /// Set up API keys
    #[command(long_about = "Set up API keys\n\nSaves API keys to ~/.config/xcli/keys.json.\nPass keys as arguments or omit them for interactive prompts.\n\nExamples:\n  xcli auth setup --api-key KEY --api-secret SECRET\n  xcli auth setup --api-key KEY --api-secret SECRET --access-token TOKEN --access-token-secret TOKEN_SECRET\n  xcli auth setup   (interactive)")]
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
        Commands::Tweet { text, dry_run } => {
            let chunks = thread::split_text(&text);

            if dry_run {
                if chunks.len() == 1 {
                    println!(
                        "Tweet preview ({}/280):\n  {}",
                        thread::weighted_len(&chunks[0]),
                        chunks[0]
                    );
                } else {
                    println!("Thread preview ({} tweets):", chunks.len());
                    for (i, chunk) in chunks.iter().enumerate() {
                        println!(
                            "  [{}/{}] ({}/280) {}",
                            i + 1,
                            chunks.len(),
                            thread::weighted_len(chunk),
                            chunk
                        );
                    }
                }
                return;
            }

            if let Err((idx, len)) = thread::validate_chunks(&chunks) {
                eprintln!(
                    "Error: chunk {} exceeds 280 characters ({}/280). Cannot post.",
                    idx + 1,
                    len
                );
                eprintln!("Use --dry-run to preview the split, or use --- separators to control splitting.");
                std::process::exit(1);
            }

            let config = load_config_or_exit();

            if chunks.len() == 1 {
                match api::create_tweet(&config, &chunks[0], None).await {
                    Ok(id) => println!("Tweet posted! ID: {id}"),
                    Err(e) => {
                        eprintln!("Failed to post tweet: {e}");
                        std::process::exit(1);
                    }
                }
            } else {
                match api::create_thread(&config, &chunks).await {
                    Ok(ids) => {
                        println!("Thread posted! ({} tweets)", ids.len());
                        for (i, id) in ids.iter().enumerate() {
                            println!("  [{}/{}] ID: {id}", i + 1, ids.len());
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "Thread failed at tweet [{}/{}]: {}",
                            e.failed_index + 1,
                            chunks.len(),
                            e.error
                        );
                        if !e.posted_ids.is_empty() {
                            eprintln!("Already posted:");
                            for (i, id) in e.posted_ids.iter().enumerate() {
                                eprintln!("  [{}/{}] ID: {id}", i + 1, chunks.len());
                            }
                        }
                        std::process::exit(1);
                    }
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
