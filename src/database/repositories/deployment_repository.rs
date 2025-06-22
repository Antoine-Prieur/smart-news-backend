use log::{error, info};
use mongodb::Collection;
use mongodb::bson::{doc, oid::ObjectId};

use crate::database::mongo_client::DatabaseClient;

use super::models::deployment_repository_models::DeploymentDocument;

#[derive(Clone)]
pub struct DeploymentRepository {
    collection: Collection<DeploymentDocument>,
}

impl DeploymentRepository {
    pub fn new(db_client: &DatabaseClient, collection_name: &str) -> Self {
        let collection: Collection<DeploymentDocument> =
            db_client.get_database().collection(collection_name);

        info!(
            "Created DeploymentRepository for collection: {}",
            collection_name
        );

        Self { collection }
    }

    pub async fn find_by_prediction_type(
        &self,
        prediction_type: &str,
    ) -> Result<Vec<DeploymentDocument>, mongodb::error::Error> {
        let filter = doc! { "prediction_type": prediction_type };

        let mut cursor = self.collection.find(filter).await?;
        let mut deployments = Vec::new();

        while cursor.advance().await? {
            match cursor.deserialize_current() {
                Ok(deployment) => deployments.push(deployment),
                Err(e) => {
                    error!("Failed to deserialize deployment: {}", e);
                    return Err(e);
                }
            }
        }

        info!(
            "Found {} deployments for prediction type '{}'",
            deployments.len(),
            prediction_type
        );
        Ok(deployments)
    }

    pub async fn find_by_id(
        &self,
        id: ObjectId,
    ) -> Result<Option<DeploymentDocument>, mongodb::error::Error> {
        let filter = doc! { "_id": id };

        match self.collection.find_one(filter).await? {
            Some(deployment) => {
                info!("Found deployment with ID: {}", id);
                Ok(Some(deployment))
            }
            none => {
                info!("No deployment found with ID: {}", id);
                Ok(None)
            }
        }
    }
}
