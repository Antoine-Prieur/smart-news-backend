use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ActiveDeploymentDocument {
    pub predictor_id: ObjectId,
    pub traffic_percentage: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeploymentDocument {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub prediction_type: String,

    pub active_deployments: Vec<ActiveDeploymentDocument>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
