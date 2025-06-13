use super::handlers;
use crate::database::ArticleRepository;
use axum::{Router, routing::get};

pub fn create_router(repository: ArticleRepository) -> Router {
    Router::new()
        .route("/articles", get(handlers::get_articles))
        .with_state(repository)
}
