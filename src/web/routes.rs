use super::handlers;
use crate::services::article_service::ArticleService;
use crate::services::metrics_service::MetricsService;
use crate::services::predictor_service::PredictorService;
use axum::{Router, routing::get};
use http::Method;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
pub struct AppState {
    pub article_service: ArticleService,
    pub metrics_service: MetricsService,
    pub predictor_service: PredictorService,
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
        .route(
            "/metrics/bins",
            get(handlers::metrics_handlers::get_metric_bins_aggregation),
        )
        .route(
            "/metrics/summary",
            get(handlers::metrics_handlers::get_metric_summary_aggregation),
        )
        .route(
            "/predictors/types",
            get(handlers::predictor_handlers::get_prediction_types),
        )
        .route(
            "/predictors/versions",
            get(handlers::predictor_handlers::get_predictor_versions),
        )
        .layer(cors)
        .with_state(app_state)
}
