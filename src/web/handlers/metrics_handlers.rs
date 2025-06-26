use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};

use crate::database::repositories::models::metrics_repository_models::MetricsDocument;
use crate::web::routes::AppState;

#[derive(Deserialize)]
pub struct MetricsQuery {
    pub metric_name: Option<String>,
    pub limit: Option<i64>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Serialize)]
pub struct MetricsResponse {
    pub metrics: Vec<MetricsDocument>,
    pub count: usize,
}

#[derive(Serialize)]
pub struct MetricAggregationResponse {
    pub metric_name: String,
    pub avg_value: f64,
    pub sum_value: f64,
    pub count: i64,
    pub min_value: f64,
    pub max_value: f64,
}

pub async fn get_metrics(
    Query(params): Query<MetricsQuery>,
    State(app_state): State<AppState>,
) -> Result<Json<MetricsResponse>, StatusCode> {
    match app_state
        .metrics_service
        .get_metrics(params.metric_name.as_deref(), params.limit)
        .await
    {
        Ok(metrics) => {
            let response = MetricsResponse {
                count: metrics.len(),
                metrics,
            };
            Ok(Json(response))
        }
        Err(e) => {
            log::error!("Service error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_metric_aggregation(
    Query(params): Query<MetricsQuery>,
    State(app_state): State<AppState>,
) -> Result<Json<MetricAggregationResponse>, StatusCode> {
    let start_time = if let Some(start_str) = params.start_time {
        match start_str.parse::<chrono::DateTime<chrono::Utc>>() {
            Ok(dt) => Some(dt),
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        }
    } else {
        None
    };

    let end_time = if let Some(end_str) = params.end_time {
        match end_str.parse::<chrono::DateTime<chrono::Utc>>() {
            Ok(dt) => Some(dt),
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        }
    } else {
        None
    };

    let metric_name = match params.metric_name {
        Some(name) => name,
        _none => return Err(StatusCode::BAD_REQUEST),
    };

    match app_state
        .metrics_service
        .get_metric_aggregation(&metric_name, start_time, end_time)
        .await
    {
        Ok(Some(aggregation)) => {
            let response = MetricAggregationResponse {
                metric_name,
                avg_value: aggregation.avg_value,
                sum_value: aggregation.sum_value,
                count: aggregation.count,
                min_value: aggregation.min_value,
                max_value: aggregation.max_value,
            };
            Ok(Json(response))
        }
        Ok(_none) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            log::error!("Service error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
