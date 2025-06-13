use super::handlers;
use crate::database::ArticleRepository;
use axum::{Router, routing::get};

pub fn create_router(repository: ArticleRepository) -> Router {
    Router::new()
        .route("/articles", get(handlers::articles_handlers::get_articles))
        .route("/health", get(handlers::health_handlers::health_check))
        .with_state(repository)
}
