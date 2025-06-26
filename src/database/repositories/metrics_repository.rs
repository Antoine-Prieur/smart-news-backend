use chrono::Utc;
use log::info;
use mongodb::Collection;
use mongodb::bson::doc;

use crate::database::mongo_client::DatabaseClient;
use crate::database::repositories::models::metrics_repository_models::MetricBinsAggregation;

use super::models::metrics_repository_models::{MetricSummaryAggregation, MetricsDocument};

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
        prediction_type: Option<&str>,
        predictor_version: Option<i32>,
        num_days: Option<i32>,
    ) -> Result<Option<MetricSummaryAggregation>, mongodb::error::Error> {
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

        if let Some(name) = predictor_version {
            match_doc.insert("tags.predictor_version", name);
        }

        let pipeline = vec![
            doc! { "$match": match_doc },
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
            let aggregation = MetricSummaryAggregation {
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
        predictor_version: Option<i32>,
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

        if let Some(name) = predictor_version {
            match_doc.insert("tags.predictor_version", name);
        }

        let pipeline = vec![
            doc! {
                "$match": match_doc
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
                    ],
                    // Generate all possible bins
                    "all_bins": [
                        {
                            "$limit": 1
                        },
                        {
                            "$project": {
                                "bins": { "$range": [0, num_bins] }
                            }
                        },
                        {
                            "$unwind": "$bins"
                        },
                        {
                            "$project": {
                                "bin_index": "$bins"
                            }
                        }
                    ]
                }
            },
            doc! {
                "$project": {
                    "stats": { "$arrayElemAt": ["$stats", 0] },
                    "data": "$data",
                    "all_bins": "$all_bins"
                }
            },
            // Calculate bin_size once
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
            // Process actual data to get counts per bin
            doc! {
                "$facet": {
                    "actual_bins": [
                        {
                            "$unwind": "$data"
                        },
                        {
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
                        {
                            "$group": {
                                "_id": "$bin_index",
                                "count": { "$sum": 1 }
                            }
                        }
                    ],
                    "bin_metadata": [
                        {
                            "$project": {
                                "stats": 1,
                                "bin_size": 1,
                                "all_bins": 1
                            }
                        }
                    ]
                }
            },
            doc! {
                "$project": {
                    "actual_bins": 1,
                    "metadata": { "$arrayElemAt": ["$bin_metadata", 0] }
                }
            },
            doc! {
                "$unwind": "$metadata.all_bins"
            },
            // Left join all possible bins with actual data
            doc! {
                "$addFields": {
                    "bin_index": "$metadata.all_bins.bin_index",
                    "count": {
                        "$let": {
                            "vars": {
                                "matching_bin": {
                                    "$arrayElemAt": [
                                        {
                                            "$filter": {
                                                "input": "$actual_bins",
                                                "cond": { "$eq": ["$$this._id", "$metadata.all_bins.bin_index"] }
                                            }
                                        },
                                        0
                                    ]
                                }
                            },
                            "in": { "$ifNull": ["$$matching_bin.count", 0] }
                        }
                    }
                }
            },
            // Calculate bin ranges
            doc! {
                "$addFields": {
                    "bin_start": {
                        "$add": [
                            "$metadata.stats.min_value",
                            { "$multiply": ["$bin_index", "$metadata.bin_size"] }
                        ]
                    },
                    "bin_end": {
                        "$add": [
                            "$metadata.stats.min_value",
                            { "$multiply": [{ "$add": ["$bin_index", 1] }, "$metadata.bin_size"] }
                        ]
                    }
                }
            },
            doc! {
                "$sort": { "bin_index": 1 }
            },
            doc! {
                "$project": {
                    "_id": 0,
                    "bin_index": 1,
                    "bin_start": { "$round": ["$bin_start", 4] },
                    "bin_end": { "$round": ["$bin_end", 4] },
                    "count": 1
                }
            },
        ];

        let mut cursor = self.collection.aggregate(pipeline).await?;
        let mut histogram_bins = Vec::new();

        while cursor.advance().await? {
            let doc = cursor.current();
            let bin = MetricBinsAggregation {
                bin_index: doc.get_i32("bin_index").unwrap_or(0),
                bin_start: doc.get_f64("bin_start").unwrap_or(0.0),
                bin_end: doc.get_f64("bin_end").unwrap_or(0.0),
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
