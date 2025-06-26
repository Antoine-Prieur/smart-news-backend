use super::handlers;
use crate::services::article_service::ArticleService;
use crate::services::metrics_service::MetricsService;
use axum::{Router, routing::get};
use http::Method;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
pub struct AppState {
    pub article_service: ArticleService,
    pub metrics_service: MetricsService,
}

pub fn create_router(app_state: AppState) -> Router {
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
        .route("/metrics", get(handlers::metrics_handlers::get_metrics))
        .route(
            "/metrics/aggregation",
            get(handlers::metrics_handlers::get_metric_aggregation),
        )
        .layer(cors)
        .with_state(app_state)
}
