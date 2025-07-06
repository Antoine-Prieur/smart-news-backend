use log::info;
use mongodb::Collection;

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
}
