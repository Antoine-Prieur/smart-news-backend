use crate::database::repositories::metrics_repository::{MetricAggregation, MetricsRepository};
use crate::database::repositories::models::metrics_repository_models::MetricsDocument;
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

    pub async fn get_metrics(
        &self,
        metric_name: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<MetricsDocument>, Box<dyn std::error::Error>> {
        info!("Getting metrics with optional filter");

        let metrics = if let Some(name) = metric_name {
            self.metrics_repository
                .find_by_metric_name(name, limit)
                .await
                .map_err(|e| {
                    error!("Failed to get metrics by name '{}': {}", name, e);
                    Box::new(e) as Box<dyn std::error::Error>
                })?
        } else {
            info!("No metric name specified, returning empty result");
            Vec::new()
        };

        info!("Successfully retrieved {} metrics", metrics.len());
        Ok(metrics)
    }

    pub async fn get_metric_aggregation(
        &self,
        metric_name: &str,
        start_time: Option<chrono::DateTime<chrono::Utc>>,
        end_time: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Option<MetricAggregation>, Box<dyn std::error::Error>> {
        info!("Getting metric aggregation for '{}'", metric_name);

        let aggregation = self
            .metrics_repository
            .get_metric_aggregation(metric_name, start_time, end_time)
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
}
