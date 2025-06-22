use log::info;
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

    pub async fn find_by_article_id_and_prediction_type(
        &self,
        article_id: ObjectId,
        prediction_type: &str,
    ) -> Result<Option<ArticlePredictionsDocument>, mongodb::error::Error> {
        let filter = doc! {
            "article_id": article_id,
            "prediction_type": prediction_type
        };

        match self.collection.find_one(filter).await? {
            Some(prediction) => {
                info!(
                    "Found prediction for article {} with type '{}'",
                    article_id, prediction_type
                );
                Ok(Some(prediction))
            }
            _none => {
                info!(
                    "No prediction found for article {} with type '{}'",
                    article_id, prediction_type
                );
                Ok(None)
            }
        }
    }
}
