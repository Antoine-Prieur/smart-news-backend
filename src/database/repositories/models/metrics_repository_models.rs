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

    pub tags: HashMap<String, String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
