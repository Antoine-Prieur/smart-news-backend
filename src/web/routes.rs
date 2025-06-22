use super::handlers;
use crate::services::article_service::ArticleService;
use axum::{Router, routing::get};
use http::Method;
use tower_http::cors::{Any, CorsLayer};

pub fn create_router(article_service: ArticleService) -> Router {
    let cors = CorsLayer::new()
        .allow_origin([
            "http://localhost:3000".parse().unwrap(),
            "https://smart-news-frontend.vercel.app".parse().unwrap(),
        ])
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    Router::new()
        .route("/articles", get(handlers::articles_handlers::get_articles))
        .route("/health", get(handlers::health_handlers::health_check))
        .layer(cors)
        .with_state(article_service)
}
