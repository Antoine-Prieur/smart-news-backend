use log::{error, info};
use mongodb::Collection;
use mongodb::bson::{doc, oid::ObjectId};

use crate::database::mongo_client::DatabaseClient;

use super::models::article_prediction_repository_models::ArticlePredictionsDocument;

#[derive(Clone)]
pub struct ArticlePredictionsRepository {
    collection: Collection<ArticlePredictionsDocument>,
}

impl ArticlePredictionsRepository {
    pub fn new(db_client: &DatabaseClient, collection_name: &str) -> Self {
        let collection: Collection<ArticlePredictionsDocument> =
            db_client.get_database().collection(collection_name);

        info!(
            "Created ArticlePredictionsRepository for collection: {}",
            collection_name
        );

        Self { collection }
    }

    pub async fn find_by_article_id(
        &self,
        article_id: ObjectId,
    ) -> Result<Vec<ArticlePredictionsDocument>, mongodb::error::Error> {
        let filter = doc! { "article_id": article_id };

        let mut cursor = self.collection.find(filter).await?;
        let mut predictions = Vec::new();

        while cursor.advance().await? {
            match cursor.deserialize_current() {
                Ok(prediction) => predictions.push(prediction),
                Err(e) => {
                    error!("Failed to deserialize prediction: {}", e);
                    return Err(e);
                }
            }
        }

        info!(
            "Found {} predictions for article {}",
            predictions.len(),
            article_id
        );
        Ok(predictions)
    }
}
