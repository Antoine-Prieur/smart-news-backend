use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MetricsDocument {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub metric_name: String,
    pub metric_value: f64,
    pub description: Option<String>,

    pub tags: HashMap<String, String>,

    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct MetricSummaryAggregation {
    pub avg_value: f64,
    pub sum_value: f64,
    pub count: i64,
    pub min_value: f64,
    pub max_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricBinsAggregation {
    pub bin_index: i32,
    pub bin_start: f64,
    pub bin_end: f64,
    pub count: i64,
}
