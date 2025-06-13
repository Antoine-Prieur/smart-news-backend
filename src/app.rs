use crate::config::Config;
use crate::database::ArticleRepository;
use crate::web::routes;
use axum::Router;
use log::{error, info};

pub struct App {
    pub router: Router,
}
impl App {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = Config::new().map_err(|e| {
            error!("Failed to load configuration: {}", e);
            e
        })?;

        let repository = ArticleRepository::new(
            &config.mongodb_connection_string,
            &config.mongodb_database_name,
            &config.mongodb_collection_name,
        )
        .await
        .map_err(|e| {
            error!("Failed to connect to MongoDB: {}", e);
            e
        })?;

        let router = routes::create_router(repository);

        info!("Application initialized successfully");

        Ok(Self { router })
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

        info!("Server starting on http://0.0.0.0:3000");

        axum::serve(listener, self.router).await?;

        Ok(())
    }
}
