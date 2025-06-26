use crate::database::repositories::metrics_repository::MetricsRepository;
use crate::database::repositories::models::metrics_repository_models::{
    MetricBinsAggregation, MetricSummaryAggregation,
};
use log::{error, info};

#[derive(Clone)]
pub struct MetricsService {
    metrics_repository: MetricsRepository,
}

impl MetricsService {
    pub fn new(metrics_repository: MetricsRepository) -> Self {
        info!("Created MetricsService");
        Self { metrics_repository }
    }

    pub async fn get_metric_aggregation(
        &self,
        metric_name: &str,
        prediction_type: Option<&str>,
        predictor_version: Option<i32>,
        num_days: Option<i32>,
    ) -> Result<Option<MetricSummaryAggregation>, Box<dyn std::error::Error>> {
        info!("Getting metric aggregation for '{}'", metric_name);

        let aggregation = self
            .metrics_repository
            .get_metric_summary_aggregation(
                metric_name,
                prediction_type,
                predictor_version,
                num_days,
            )
            .await
            .map_err(|e| {
                error!(
                    "Failed to get metric aggregation for '{}': {}",
                    metric_name, e
                );
                Box::new(e) as Box<dyn std::error::Error>
            })?;

        if aggregation.is_some() {
            info!("Successfully calculated aggregation for '{}'", metric_name);
        } else {
            info!("No data found for metric '{}'", metric_name);
        }

        Ok(aggregation)
    }

    pub async fn get_metric_bins_aggregation(
        &self,
        metric_name: &str,
        num_bins: i32,
        prediction_type: Option<&str>,
        predictor_version: Option<i32>,
        num_days: Option<i32>,
    ) -> Result<Vec<MetricBinsAggregation>, Box<dyn std::error::Error>> {
        let aggregation = self
            .metrics_repository
            .get_metric_bins_aggregation(
                metric_name,
                num_bins,
                prediction_type,
                predictor_version,
                num_days,
            )
            .await
            .map_err(|e| {
                error!(
                    "Failed to get metric bin aggregation for '{}': {}",
                    metric_name, e
                );
                Box::new(e) as Box<dyn std::error::Error>
            })?;

        info!(
            "Successfully calculated aggregation bin for '{}'",
            metric_name
        );

        Ok(aggregation)
    }
}
