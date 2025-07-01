use log::{error, info};
use std::collections::HashSet;

use crate::database::repositories::predictors_repository::PredictorRepository;

#[derive(Clone)]
pub struct PredictorService {
    predictor_repository: PredictorRepository,
}

impl PredictorService {
    pub fn new(predictor_repository: PredictorRepository) -> Self {
        info!("Created PredictorService");
        Self {
            predictor_repository,
        }
    }

    pub async fn get_prediction_types(
        &self,
    ) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
        info!("Getting all prediction types");

        let prediction_types = self
            .predictor_repository
            .get_prediction_types()
            .await
            .map_err(|e| {
                error!("Failed to get prediction types: {}", e);
                Box::new(e) as Box<dyn std::error::Error>
            })?;

        info!(
            "Successfully retrieved {} prediction types",
            prediction_types.len()
        );

        Ok(prediction_types)
    }

    pub async fn get_predictor_versions(
        &self,
        prediction_type: &str,
    ) -> Result<HashSet<i32>, Box<dyn std::error::Error>> {
        info!(
            "Getting predictor versions for prediction type '{}'",
            prediction_type
        );

        let predictor_versions = self
            .predictor_repository
            .get_predictor_versions(prediction_type)
            .await
            .map_err(|e| {
                error!(
                    "Failed to get predictor versions for '{}': {}",
                    prediction_type, e
                );
                Box::new(e) as Box<dyn std::error::Error>
            })?;

        info!(
            "Successfully retrieved {} predictor versions for prediction type '{}'",
            predictor_versions.len(),
            prediction_type
        );

        Ok(predictor_versions)
    }

    pub async fn get_predictors_by_type(
        &self,
        prediction_type: &str,
        min_traffic: Option<i32>,
    ) -> Result<
        Vec<crate::database::repositories::models::predictor_repository_models::PredictorDocument>,
        Box<dyn std::error::Error>,
    > {
        info!(
            "Getting all predictors for prediction type '{}'",
            prediction_type
        );

        let predictors = self
            .predictor_repository
            .get_predictors_by_type(prediction_type, min_traffic)
            .await
            .map_err(|e| {
                error!("Failed to get predictors for '{}': {}", prediction_type, e);
                Box::new(e) as Box<dyn std::error::Error>
            })?;

        info!(
            "Successfully retrieved {} predictors for prediction type '{}'",
            predictors.len(),
            prediction_type
        );

        Ok(predictors)
    }
}
