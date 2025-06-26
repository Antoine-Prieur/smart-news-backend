use crate::config::Config;
use crate::database::mongo_client::DatabaseClient;
use crate::database::repositories::deployment_repository::DeploymentRepository;
use crate::database::repositories::metrics_repository::MetricsRepository;
use crate::database::{ArticlePredictionsRepository, ArticleRepository};
use crate::services::article_service::ArticleService;
use crate::services::metrics_service::MetricsService;
use crate::web::routes::{self, AppState};
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

        // Create all repositories
        let articles_repository =
            ArticleRepository::new(&db_client, &config.articles_collection_name);

        let _article_predictions_repository = ArticlePredictionsRepository::new(
            &db_client,
            &config.article_predictions_collection_name,
        );

        let _deployment_repository =
            DeploymentRepository::new(&db_client, &config.deployment_collection_name);

        let metrics_repository =
            MetricsRepository::new(&db_client, &config.metrics_collection_name);

        // Create services
        let article_service = ArticleService::new(articles_repository);
        let metrics_service = MetricsService::new(metrics_repository);

        // Create app state with both services
        let app_state = AppState {
            article_service,
            metrics_service,
        };

        let router = routes::create_router(app_state);

        info!("Application initialized successfully");

        Ok(Self { router })
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;

        info!("Server starting on http://0.0.0.0:8000");

        axum::serve(listener, self.router).await?;

        Ok(())
    }
}
