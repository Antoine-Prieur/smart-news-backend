use chrono::Utc;
use log::info;
use mongodb::Collection;
use mongodb::bson::doc;

use crate::database::mongo_client::DatabaseClient;
use crate::database::repositories::models::metrics_repository_models::MetricBinsAggregation;

use super::models::metrics_repository_models::{MetricAggregation, MetricsDocument};

#[derive(Clone)]
pub struct MetricsRepository {
    collection: Collection<MetricsDocument>,
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

    pub async fn get_metric_summary_aggregation(
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
                count: doc
                    .get_i32("count")
                    .map(|v| v as i64)
                    .unwrap_or_else(|_| doc.get_i64("count").unwrap_or(0)),
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

    pub async fn get_metric_bins_aggregation(
        &self,
        metric_name: &str,
        num_bins: i32,
        prediction_type: Option<&str>,
        num_days: Option<i32>,
    ) -> Result<Vec<MetricBinsAggregation>, mongodb::error::Error> {
        let start_time = Utc::now() - chrono::Duration::days(num_days.unwrap_or(7) as i64);

        let mut match_doc = doc! {
            "created_at": {
                "$gte": start_time
            },
            "metric_name": metric_name
        };

        if let Some(name) = prediction_type {
            match_doc.insert("tags.prediction_type", name);
        }

        let pipeline = vec![
            doc! {
            "$match":  match_doc

            },
            doc! {
                "$facet": {
                    "stats": [
                        {
                            "$group": {
                                "_id": null,
                                "min_value": { "$min": "$metric_value" },
                                "max_value": { "$max": "$metric_value" }
                            }
                        }
                    ],
                    "data": [
                        { "$project": { "metric_value": 1 } }
                    ]
                }
            },
            doc! {
                "$project": {
                    "stats": { "$arrayElemAt": ["$stats", 0] },
                    "data": "$data"
                }
            },
            doc! {
                "$unwind": "$data"
            },
            doc! {
                "$addFields": {
                    "bin_size": {
                        "$divide": [
                            { "$subtract": ["$stats.max_value", "$stats.min_value"] },
                            num_bins
                        ]
                    }
                }
            },
            doc! {
                "$addFields": {
                    "bin_index": {
                        "$min": [
                            {
                                "$floor": {
                                    "$divide": [
                                        { "$subtract": ["$data.metric_value", "$stats.min_value"] },
                                        "$bin_size"
                                    ]
                                }
                            },
                            num_bins - 1
                        ]
                    }
                }
            },
            doc! {
                "$addFields": {
                    "bin_start": {
                        "$add": [
                            "$stats.min_value",
                            { "$multiply": ["$bin_index", "$bin_size"] }
                        ]
                    },
                    "bin_end": {
                        "$add": [
                            "$stats.min_value",
                            { "$multiply": [{ "$add": ["$bin_index", 1] }, "$bin_size"] }
                        ]
                    }
                }
            },
            doc! {
                "$group": {
                    "_id": "$bin_index",
                    "bin_start": { "$first": "$bin_start" },
                    "bin_end": { "$first": "$bin_end" },
                    "count": { "$sum": 1 }
                }
            },
            doc! {
                "$sort": { "_id": 1 }
            },
            doc! {
                "$project": {
                    "_id": 0,
                    "bin_index": "$_id",
                    "bin_range": {
                        "$concat": [
                            "[",
                            { "$toString": { "$round": ["$bin_start", 4] } },
                            ", ",
                            { "$toString": { "$round": ["$bin_end", 4] } },
                            ")"
                        ]
                    },
                    "count": 1
                }
            },
        ];

        let mut cursor = self.collection.aggregate(pipeline).await?;
        let mut histogram_bins = Vec::new();

        while cursor.advance().await? {
            let doc = cursor.current();
            let bin = MetricBinsAggregation {
                bin_index: doc.get_f64("bin_index").unwrap_or(0.0) as i32,
                bin_range: doc.get_str("bin_range").unwrap_or("").to_string(),
                count: doc
                    .get_i32("count")
                    .map(|v| v as i64)
                    .unwrap_or_else(|_| doc.get_i64("count").unwrap_or(0)),
            };
            histogram_bins.push(bin);
        }

        info!(
            "Generated histogram with {} bins for metric '{}' and predictor_type '{}' over {} days",
            histogram_bins.len(),
            metric_name,
            prediction_type.unwrap_or(""),
            num_days.unwrap_or(7)
        );

        Ok(histogram_bins)
    }
}
