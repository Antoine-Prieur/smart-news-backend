use log::info;
use mongodb::Collection;
use mongodb::bson::doc;
use std::collections::HashSet;

use crate::database::mongo_client::DatabaseClient;

use super::models::predictor_repository_models::PredictorDocument;

#[derive(Clone)]
pub struct PredictorRepository {
    collection: Collection<PredictorDocument>,
}

impl PredictorRepository {
    pub fn new(db_client: &DatabaseClient, collection_name: &str) -> Self {
        let collection: Collection<PredictorDocument> =
            db_client.get_database().collection(collection_name);

        info!(
            "Created PredictorRepository for collection: {}",
            collection_name
        );

        Self { collection }
    }

    pub async fn get_prediction_types(&self) -> Result<HashSet<String>, mongodb::error::Error> {
        let pipeline = vec![
            doc! {
                "$group": {
                    "_id": "$prediction_type"
                }
            },
            doc! {
                "$sort": { "_id": 1 }
            },
        ];

        let mut cursor = self.collection.aggregate(pipeline).await?;
        let mut prediction_types = HashSet::new();

        while cursor.advance().await? {
            let doc = cursor.current();
            if let Ok(prediction_type) = doc.get_str("_id") {
                prediction_types.insert(prediction_type.to_string());
            }
        }

        info!("Found {} unique prediction types", prediction_types.len());

        Ok(prediction_types)
    }

    pub async fn get_predictor_versions(
        &self,
        prediction_type: &str,
    ) -> Result<HashSet<i32>, mongodb::error::Error> {
        let pipeline = vec![
            doc! {
                "$match": {
                    "prediction_type": prediction_type
                }
            },
            doc! {
                "$group": {
                    "_id": "$predictor_version"
                }
            },
            doc! {
                "$sort": { "_id": 1 }
            },
        ];

        let mut cursor = self.collection.aggregate(pipeline).await?;
        let mut predictor_versions = HashSet::new();

        while cursor.advance().await? {
            let doc = cursor.current();
            if let Ok(version) = doc.get_i32("_id") {
                predictor_versions.insert(version);
            }
        }

        info!(
            "Found {} unique predictor versions for prediction type '{}'",
            predictor_versions.len(),
            prediction_type
        );

        Ok(predictor_versions)
    }

    pub async fn get_predictors_by_type(
        &self,
        prediction_type: &str,
        min_traffic: Option<i32>,
    ) -> Result<Vec<PredictorDocument>, mongodb::error::Error> {
        let mut filter = doc! { "prediction_type": prediction_type };

        if let Some(min_traffic_value) = min_traffic {
            filter.insert("traffic_percentage", doc! { "$gte": min_traffic_value });
        }

        let mut options = mongodb::options::FindOptions::default();
        options.sort = Some(doc! { "predictor_version": 1 });

        let mut cursor = self
            .collection
            .find(filter)
            .with_options(Some(options))
            .await?;

        let mut predictors = Vec::new();

        while cursor.advance().await? {
            match cursor.deserialize_current() {
                Ok(predictor) => predictors.push(predictor),
                Err(e) => {
                    log::error!("Failed to deserialize predictor: {}", e);
                    return Err(e);
                }
            }
        }

        info!(
            "Found {} predictors for prediction type '{}'",
            predictors.len(),
            prediction_type
        );

        Ok(predictors)
    }
}
