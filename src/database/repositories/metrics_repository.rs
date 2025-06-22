use log::{error, info};
use mongodb::Collection;
use mongodb::bson::{doc, oid::ObjectId};
use std::collections::HashMap;

use crate::database::mongo_client::DatabaseClient;

use super::models::metrics_repository_models::MetricsDocument;

#[derive(Clone)]
pub struct MetricsRepository {
    collection: Collection<MetricsDocument>,
}

#[derive(Debug)]
pub struct MetricAggregation {
    pub avg_value: f64,
    pub sum_value: f64,
    pub count: i64,
    pub min_value: f64,
    pub max_value: f64,
}

impl MetricsRepository {
    pub fn new(db_client: &DatabaseClient, collection_name: &str) -> Self {
        let collection: Collection<MetricsDocument> =
            db_client.get_database().collection(collection_name);

        info!(
            "Created MetricsRepository for collection: {}",
            collection_name
        );

        Self { collection }
    }

    pub async fn find_by_metric_name(
        &self,
        metric_name: &str,
        limit: Option<i64>,
    ) -> Result<Vec<MetricsDocument>, mongodb::error::Error> {
        let filter = doc! { "metric_name": metric_name };

        let mut options = mongodb::options::FindOptions::default();
        if let Some(limit_val) = limit {
            options.limit = Some(limit_val);
        }
        options.sort = Some(doc! { "created_at": -1 });

        let mut cursor = self
            .collection
            .find(filter)
            .with_options(Some(options))
            .await?;
        let mut metrics = Vec::new();

        while cursor.advance().await? {
            match cursor.deserialize_current() {
                Ok(metric) => metrics.push(metric),
                Err(e) => {
                    error!("Failed to deserialize metric: {}", e);
                    return Err(e);
                }
            }
        }

        info!(
            "Found {} metrics with name '{}'",
            metrics.len(),
            metric_name
        );
        Ok(metrics)
    }

    pub async fn find_by_tags(
        &self,
        tags: HashMap<String, String>,
        limit: Option<i64>,
    ) -> Result<Vec<MetricsDocument>, mongodb::error::Error> {
        // Build filter for tags - MongoDB stores them as a subdocument
        let mut filter = doc! {};
        for (key, value) in tags {
            filter.insert(format!("tags.{}", key), value);
        }

        let mut options = mongodb::options::FindOptions::default();
        if let Some(limit_val) = limit {
            options.limit = Some(limit_val);
        }
        options.sort = Some(doc! { "created_at": -1 });

        let mut cursor = self
            .collection
            .find(filter)
            .with_options(Some(options))
            .await?;
        let mut metrics = Vec::new();

        while cursor.advance().await? {
            match cursor.deserialize_current() {
                Ok(metric) => metrics.push(metric),
                Err(e) => {
                    error!("Failed to deserialize metric: {}", e);
                    return Err(e);
                }
            }
        }

        info!(
            "Found {} metrics matching the specified tags",
            metrics.len()
        );
        Ok(metrics)
    }

    pub async fn find_by_id(
        &self,
        id: ObjectId,
    ) -> Result<Option<MetricsDocument>, mongodb::error::Error> {
        let filter = doc! { "_id": id };

        match self.collection.find_one(filter).await? {
            Some(metric) => {
                info!("Found metric with ID: {}", id);
                Ok(Some(metric))
            }
            None => {
                info!("No metric found with ID: {}", id);
                Ok(None)
            }
        }
    }

    pub async fn find_by_time_range(
        &self,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
        metric_name: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<MetricsDocument>, mongodb::error::Error> {
        let mut filter = doc! {
            "created_at": {
                "$gte": start_time,
                "$lte": end_time
            }
        };

        if let Some(name) = metric_name {
            filter.insert("metric_name", name);
        }

        let mut options = mongodb::options::FindOptions::default();
        if let Some(limit_val) = limit {
            options.limit = Some(limit_val);
        }
        options.sort = Some(doc! { "created_at": -1 });

        let mut cursor = self
            .collection
            .find(filter)
            .with_options(Some(options))
            .await?;
        let mut metrics = Vec::new();

        while cursor.advance().await? {
            match cursor.deserialize_current() {
                Ok(metric) => metrics.push(metric),
                Err(e) => {
                    error!("Failed to deserialize metric: {}", e);
                    return Err(e);
                }
            }
        }

        info!(
            "Found {} metrics in time range {} to {}",
            metrics.len(),
            start_time,
            end_time
        );
        Ok(metrics)
    }

    pub async fn get_metric_aggregation(
        &self,
        metric_name: &str,
        start_time: Option<chrono::DateTime<chrono::Utc>>,
        end_time: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Option<MetricAggregation>, mongodb::error::Error> {
        let mut match_stage = doc! { "metric_name": metric_name };

        if let (Some(start), Some(end)) = (start_time, end_time) {
            match_stage.insert(
                "created_at",
                doc! {
                    "$gte": start,
                    "$lte": end
                },
            );
        }

        let pipeline = vec![
            doc! { "$match": match_stage },
            doc! {
                "$group": {
                    "_id": null,
                    "avg_value": { "$avg": "$metric_value" },
                    "sum_value": { "$sum": "$metric_value" },
                    "count": { "$sum": 1 },
                    "min_value": { "$min": "$metric_value" },
                    "max_value": { "$max": "$metric_value" }
                }
            },
        ];

        let mut cursor = self.collection.aggregate(pipeline).await?;

        if cursor.advance().await? {
            let doc = cursor.current();
            let aggregation = MetricAggregation {
                avg_value: doc.get_f64("avg_value").unwrap_or(0.0),
                sum_value: doc.get_f64("sum_value").unwrap_or(0.0),
                count: doc.get_i64("count").unwrap_or(0),
                min_value: doc.get_f64("min_value").unwrap_or(0.0),
                max_value: doc.get_f64("max_value").unwrap_or(0.0),
            };

            info!("Calculated aggregation for metric '{}'", metric_name);
            Ok(Some(aggregation))
        } else {
            info!("No data found for metric '{}'", metric_name);
            Ok(None)
        }
    }
}
