mod api;
mod auth;
mod config;

use clap::{Parser, Subcommand};
use config::Config;

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
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {e}");
            eprintln!("Make sure your .env file contains X_API_KEY, X_API_SECRET, X_ACCESS_TOKEN, and X_ACCESS_TOKEN_SECRET.");
            std::process::exit(1);
        }
    };

    match cli.command {
        Commands::Tweet { text } => match api::create_tweet(&config, &text).await {
            Ok(id) => println!("Tweet posted! ID: {id}"),
            Err(e) => {
                eprintln!("Failed to post tweet: {e}");
                std::process::exit(1);
            }
        },
        Commands::Delete { id } => match api::delete_tweet(&config, &id).await {
            Ok(true) => println!("Tweet {id} deleted."),
            Ok(false) => {
                eprintln!("Tweet {id} was not deleted.");
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("Failed to delete tweet: {e}");
                std::process::exit(1);
            }
        },
    }
}
