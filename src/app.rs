use crate::config::Config;
use crate::database::mongo_client::DatabaseClient;
use crate::database::repositories::deployment_repository::DeploymentRepository;
use crate::database::repositories::metrics_repository::MetricsRepository;
use crate::database::{ArticlePredictionsRepository, ArticleRepository};
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

        let db_client = DatabaseClient::new(
            &config.mongodb_connection_string,
            &config.mongodb_database_name,
        )
        .await
        .map_err(|e| {
            error!("Failed to connect to MongoDB: {}", e);
            e
        })?;

        let articles_repository =
            ArticleRepository::new(&db_client, &config.articles_collection_name);

        let article_predictions_repository = ArticlePredictionsRepository::new(
            &db_client,
            &config.article_predictions_collection_name,
        );

        let deployment_repository =
            DeploymentRepository::new(&db_client, &config.deployment_collection_name);

        let metrics_repository =
            MetricsRepository::new(&db_client, &config.metrics_collection_name);

        let router = routes::create_router(articles_repository);

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
