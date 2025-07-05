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

    pub async fn list_metrics(
        &self,
        metric_name: &str,
        limit: Option<i64>,
        skip: Option<u64>,
        prediction_type: Option<String>,
        predictor_version: Option<String>,
    ) -> Result<Vec<MetricsDocument>, mongodb::error::Error> {
        let mut options = mongodb::options::FindOptions::default();

        options.skip = Some(skip.unwrap_or(0));
        options.limit = Some(limit.unwrap_or(20));

        options.sort = Some(doc! { "created_at": -1 });

        let mut filter = doc! {};
        filter.insert("metric_name", metric_name);

        if let Some(pred_type) = prediction_type {
            filter.insert("tags.prediction_type", pred_type);
        }

        if let Some(pred_version) = predictor_version {
            filter.insert("tags.predictor_version", pred_version);
        }

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
                    log::error!("Failed to deserialize metric: {}", e);
                    return Err(e);
                }
            }
        }

        info!("Retrieved {} metrics from database", metrics.len());

        Ok(metrics)
    }

    pub async fn get_metric_summary_aggregation(
        &self,
        metric_name: &str,
        prediction_type: Option<&str>,
        predictor_version: Option<&str>,
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
        predictor_version: Option<&str>,
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
                                "max_value": { "$max": "$metric_value" },
                                "count": { "$sum": 1 }
                            }
                        }
                    ],
                    "data": [
                        { "$project": { "metric_value": 1 } }
                    ],
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
            doc! {
                "$addFields": {
                    "bin_size": {
                        "$cond": {
                            "if": {
                                "$or": [
                                    { "$eq": ["$stats.count", 1] },
                                    { "$eq": ["$stats.min_value", "$stats.max_value"] }
                                ]
                            },
                            "then": 1.0,
                            "else": {
                                "$divide": [
                                    { "$subtract": ["$stats.max_value", "$stats.min_value"] },
                                    num_bins
                                ]
                            }
                        }
                    }
                }
            },
            doc! {
                "$facet": {
                    "actual_bins": [
                        {
                            "$unwind": "$data"
                        },
                        {
                            "$addFields": {
                                "bin_index": {
                                    "$cond": {
                                        "if": {
                                            "$or": [
                                                { "$eq": ["$stats.count", 1] },
                                                { "$eq": ["$stats.min_value", "$stats.max_value"] }
                                            ]
                                        },
                                        "then": 0,
                                        "else": {
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
            doc! {
                "$addFields": {
                    "bin_start": {
                        "$cond": {
                            "if": {
                                "$or": [
                                    { "$eq": ["$metadata.stats.count", 1] },
                                    { "$eq": ["$metadata.stats.min_value", "$metadata.stats.max_value"] }
                                ]
                            },
                            "then": "$metadata.stats.min_value",
                            "else": {
                                "$add": [
                                    "$metadata.stats.min_value",
                                    { "$multiply": ["$bin_index", "$metadata.bin_size"] }
                                ]
                            }
                        }
                    },
                    "bin_end": {
                        "$cond": {
                            "if": {
                                "$or": [
                                    { "$eq": ["$metadata.stats.count", 1] },
                                    { "$eq": ["$metadata.stats.min_value", "$metadata.stats.max_value"] }
                                ]
                            },
                            "then": "$metadata.stats.max_value",
                            "else": {
                                "$add": [
                                    "$metadata.stats.min_value",
                                    { "$multiply": [{ "$add": ["$bin_index", 1] }, "$metadata.bin_size"] }
                                ]
                            }
                        }
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
