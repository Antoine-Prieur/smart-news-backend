use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Articles
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SourceDocument {
    pub id: Option<String>,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PredictionDocument {
    pub prediction_confidence: Option<f64>,
    pub prediction_value: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ArticleDocument {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub source: SourceDocument,
    pub author: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub url_to_image: Option<String>,

    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub published_at: DateTime<Utc>,

    pub content: Option<String>,

    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,

    // New field: HashMap of prediction_type -> selected_prediction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub predictions: Option<HashMap<String, PredictionDocument>>,

    // Keep the old field for backward compatibility, but it's deprecated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sentiment_analysis: Option<PredictionDocument>,
}

#[derive(Debug, Clone)]
pub struct PaginatedArticles {
    pub articles: Vec<ArticleDocument>,
    pub total_count: u64,
    pub current_page_count: usize,
    pub page: u64,
    pub per_page: i64,
    pub total_pages: u64,
}
