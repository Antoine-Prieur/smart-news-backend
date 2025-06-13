mod config;
mod database;

use self::database::ArticleRepository;

use crate::config::Config;
use log::error;

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = match Config::new() {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    let repository = match ArticleRepository::new(
        &config.mongodb_connection_string,
        &config.mongodb_database_name,
        &config.mongodb_collection_name,
    )
    .await
    {
        Ok(repo) => repo,
        Err(e) => {
            error!("Failed to connect to MongoDB: {}", e);
            std::process::exit(1);
        }
    };
}
