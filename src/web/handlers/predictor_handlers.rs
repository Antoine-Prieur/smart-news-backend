use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    database::repositories::models::predictor_repository_models::PredictorDocument,
    web::routes::AppState,
};

#[derive(Deserialize)]
pub struct PredictorsQuery {
    pub prediction_type: Option<String>,
    pub min_traffic: Option<i32>,
}

#[derive(Deserialize)]
pub struct PredictorVersionsQuery {
    pub prediction_type: Option<String>,
}

#[derive(Serialize)]
pub struct PredictorsResponse {
    pub prediction_type: String,
    pub predictors: Vec<PredictorDocument>,
}

#[derive(Serialize)]
pub struct PredictionTypesResponse {
    pub prediction_types: Vec<String>,
}

#[derive(Serialize)]
pub struct PredictorVersionsResponse {
    pub prediction_type: String,
    pub predictor_versions: Vec<i32>,
}

pub async fn get_prediction_types(
    State(app_state): State<AppState>,
) -> Result<Json<PredictionTypesResponse>, StatusCode> {
    match app_state.predictor_service.get_prediction_types().await {
        Ok(prediction_types_set) => {
            let mut prediction_types: Vec<String> = prediction_types_set.into_iter().collect();
            prediction_types.sort();

            let response = PredictionTypesResponse { prediction_types };
            Ok(Json(response))
        }
        Err(e) => {
            log::error!("Service error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_predictor_versions(
    Query(params): Query<PredictorVersionsQuery>,
    State(app_state): State<AppState>,
) -> Result<Json<PredictorVersionsResponse>, StatusCode> {
    let prediction_type = match params.prediction_type {
        Some(prediction_type) => prediction_type,
        _none => return Err(StatusCode::BAD_REQUEST),
    };

    match app_state
        .predictor_service
        .get_predictor_versions(&prediction_type)
        .await
    {
        Ok(predictor_versions_set) => {
            // Convert HashSet to Vec and sort for consistent ordering
            let mut predictor_versions: Vec<i32> = predictor_versions_set.into_iter().collect();
            predictor_versions.sort();

            let response = PredictorVersionsResponse {
                prediction_type,
                predictor_versions,
            };
            Ok(Json(response))
        }
        Err(e) => {
            log::error!("Service error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_predictors(
    Query(params): Query<PredictorsQuery>,
    State(app_state): State<AppState>,
) -> Result<Json<PredictorsResponse>, StatusCode> {
    let prediction_type = match params.prediction_type {
        Some(prediction_type) => prediction_type,
        _none => return Err(StatusCode::BAD_REQUEST),
    };

    match app_state
        .predictor_service
        .get_predictors_by_type(&prediction_type, params.min_traffic)
        .await
    {
        Ok(predictors) => {
            let response = PredictorsResponse {
                prediction_type,
                predictors,
            };
            Ok(Json(response))
        }
        Err(e) => {
            log::error!("Service error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
