use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PredictionDocument {
    pub prediction_confidence: Option<f64>,
    pub prediction_value: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ArticlePredictionsDocument {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub article_id: ObjectId,
    pub prediction_type: String,

    pub selected_predictor_id: ObjectId,
    pub selected_prediction: PredictionDocument,

    pub predictions: HashMap<ObjectId, PredictionDocument>,

    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
}
