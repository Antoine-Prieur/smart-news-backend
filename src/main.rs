mod config;
mod database;
mod web;

use self::database::ArticleRepository;

use crate::config::Config;
use crate::web::get_articles;
use axum::{Router, routing::get};
use log::error;
use log::info;

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

    let app = Router::new()
        .route("/articles", get(get_articles))
        .with_state(repository);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    info!("Server starting on http://0.0.0.0:3000");
    info!("Try: http://localhost:3000/articles");
    info!("With params: http://localhost:3000/articles?limit=2");

    axum::serve(listener, app).await.unwrap();
}
