use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};

use crate::database::repositories::models::metrics_repository_models::MetricBinsAggregation;
use crate::web::routes::AppState;

#[derive(Deserialize)]
pub struct MetricsSummaryQuery {
    pub metric_name: Option<String>,
    pub prediction_type: Option<String>,
    pub predictor_version: Option<String>,
    pub num_days: Option<i32>,
}

#[derive(Deserialize)]
pub struct MetricBinsQuery {
    pub metric_name: Option<String>,
    pub num_bins: Option<i32>,
    pub prediction_type: Option<String>,
    pub predictor_version: Option<String>,
    pub num_days: Option<i32>,
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

#[derive(Serialize)]
pub struct MetricBinsAggregationResponse {
    pub metric_bins: Vec<MetricBinsAggregation>,
}

pub async fn get_metric_summary_aggregation(
    Query(params): Query<MetricsSummaryQuery>,
    State(app_state): State<AppState>,
) -> Result<Json<MetricAggregationResponse>, StatusCode> {
    let metric_name = match params.metric_name {
        Some(name) => name,
        _none => return Err(StatusCode::BAD_REQUEST),
    };

    match app_state
        .metrics_service
        .get_metric_aggregation(
            &metric_name,
            params.prediction_type.as_deref(),
            params.predictor_version.as_deref(),
            params.num_days,
        )
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

pub async fn get_metric_bins_aggregation(
    Query(params): Query<MetricBinsQuery>,
    State(app_state): State<AppState>,
) -> Result<Json<MetricBinsAggregationResponse>, StatusCode> {
    let metric_name = match params.metric_name {
        Some(name) => name,
        _none => return Err(StatusCode::BAD_REQUEST),
    };

    let num_bins = match params.num_bins {
        Some(num) => num,
        _none => return Err(StatusCode::BAD_REQUEST),
    };

    match app_state
        .metrics_service
        .get_metric_bins_aggregation(
            &metric_name,
            num_bins,
            params.prediction_type.as_deref(),
            params.predictor_version.as_deref(),
            params.num_days,
        )
        .await
    {
        Ok(aggregation) => {
            let response = MetricBinsAggregationResponse {
                metric_bins: aggregation,
            };
            Ok(Json(response))
        }
        Err(e) => {
            log::error!("Service error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
