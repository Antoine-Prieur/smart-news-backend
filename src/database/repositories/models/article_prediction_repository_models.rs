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
    pub selected_prediction: ObjectId,

    pub predictions: HashMap<ObjectId, PredictionDocument>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
