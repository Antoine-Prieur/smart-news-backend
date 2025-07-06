use bson::Document;
use log::{error, info};
use mongodb::Collection;
use mongodb::bson::doc;
use serde::Deserialize;

use crate::database::mongo_client::DatabaseClient;

use super::models::article_repository_models::{ArticleDocument, PaginatedArticles};

#[derive(Clone)]
pub struct ArticleRepository {
    collection: Collection<ArticleDocument>,
}

#[derive(Debug, Deserialize)]
struct FacetResult {
    data: Vec<ArticleDocument>,
    #[serde(rename = "totalCount")]
    total_count: Vec<CountResult>,
}

#[derive(Debug, Deserialize)]
struct CountResult {
    count: u64,
}

impl ArticleRepository {
    pub fn new(db_client: &DatabaseClient, collection_name: &str) -> Self {
        let collection: Collection<ArticleDocument> =
            db_client.get_database().collection(collection_name);

        info!(
            "Created ArticleRepository for collection: {}",
            collection_name
        );

        Self { collection }
    }

    pub async fn list_articles(
        &self,
        limit: Option<i64>,
        skip: Option<u64>,
    ) -> Result<PaginatedArticles, mongodb::error::Error> {
        let filter = doc! {};

        let total_count = self.collection.count_documents(filter.clone()).await?;

        let mut options = mongodb::options::FindOptions::default();
        let skip_count = skip.unwrap_or(0);
        let limit_count = limit.unwrap_or(20);

        options.skip = Some(skip_count);
        options.limit = Some(limit_count);
        options.sort = Some(doc! { "published_at": -1 });

        let mut cursor = self
            .collection
            .find(filter)
            .with_options(Some(options))
            .await?;

        let mut articles = Vec::new();

        while cursor.advance().await? {
            match cursor.deserialize_current() {
                Ok(article) => articles.push(article),
                Err(e) => {
                    error!("Failed to deserialize article: {}", e);
                    return Err(e);
                }
            }
        }

        let current_page_count = articles.len();
        let page = (skip_count / limit_count as u64) + 1;
        let total_pages = total_count.div_ceil(limit_count as u64);

        info!(
            "Retrieved {} articles from database (page {} of {})",
            current_page_count, page, total_pages
        );

        Ok(PaginatedArticles {
            articles,
            total_count,
            current_page_count,
            page,
            per_page: limit_count,
            total_pages,
        })
    }

    /// New method that gets articles with ALL predictions
    pub async fn list_articles_with_all_predictions(
        &self,
        limit: Option<i64>,
        skip: Option<u64>,
        sentiment: Option<&str>,
    ) -> Result<PaginatedArticles, mongodb::error::Error> {
        let skip_count = skip.unwrap_or(0);
        let limit_count = limit.unwrap_or(20);

        // This aggregation pipeline will:
        // 1. Look up all predictions for each article
        // 2. Convert them to a HashMap of prediction_type -> selected_prediction
        // 3. Filter by sentiment if requested
        let mut pipeline = vec![
            // Step 1: Lookup all predictions for each article
            doc! {
                "$lookup": {
                    "from": "article_predictions",
                    "let": { "articleId": "$_id" },
                    "pipeline": [
                        {
                            "$match": {
                                "$expr": { "$eq": ["$article_id", "$$articleId"] }
                            }
                        },
                        {
                            "$project": {
                                "prediction_type": 1,
                                "selected_prediction": 1
                            }
                        }
                    ],
                    "as": "all_predictions"
                }
            },
            // Step 2: Convert the array of predictions into a map
            doc! {
                "$addFields": {
                    "predictions": {
                        "$cond": {
                            "if": { "$gt": [{ "$size": "$all_predictions" }, 0] },
                            "then": {
                                "$arrayToObject": {
                                    "$map": {
                                        "input": "$all_predictions",
                                        "as": "pred",
                                        "in": {
                                            "k": "$$pred.prediction_type",
                                            "v": "$$pred.selected_prediction"
                                        }
                                    }
                                }
                            },
                            "else": null
                        }
                    },
                    // Keep sentiment_analysis for backward compatibility
                    "sentiment_analysis": {
                        "$let": {
                            "vars": {
                                "sentiment_pred": {
                                    "$arrayElemAt": [
                                        {
                                            "$filter": {
                                                "input": "$all_predictions",
                                                "cond": { "$eq": ["$$this.prediction_type", "sentiment_analysis"] }
                                            }
                                        },
                                        0
                                    ]
                                }
                            },
                            "in": "$$sentiment_pred.selected_prediction"
                        }
                    }
                }
            },
            // Step 3: Remove the temporary all_predictions field
            doc! {
                "$project": {
                    "all_predictions": 0
                }
            },
        ];

        // Step 4: Add sentiment filter if requested
        if let Some(sentiment_value) = sentiment {
            pipeline.push(doc! {
                "$match": {
                    "predictions.sentiment_analysis.prediction_value": sentiment_value
                }
            });
        }

        // Step 5: Sort by published_at
        pipeline.push(doc! {
            "$sort": { "published_at": -1 }
        });

        // Step 6: Add pagination and count
        pipeline.push(doc! {
            "$facet": {
                "data": [
                    { "$skip": skip_count as i32 },
                    { "$limit": limit_count as i32 }
                ],
                "totalCount": [
                    { "$group": { "_id": null, "count": { "$sum": 1 } } }
                ]
            }
        });

        let mut cursor = self.collection.aggregate(pipeline).await?;

        if cursor.advance().await? {
            let doc = cursor.current();
            let document: Document = doc.try_into()?;
            let facet_result: FacetResult = mongodb::bson::from_document(document)?;

            let total_count = facet_result
                .total_count
                .first()
                .map(|c| c.count)
                .unwrap_or(0);

            let current_page_count = facet_result.data.len();
            let page = (skip_count / limit_count as u64) + 1;
            let total_pages = total_count.div_ceil(limit_count as u64);

            let log_message = if let Some(sentiment_value) = sentiment {
                format!(
                    "Retrieved {} articles with sentiment '{}' and all predictions from database (page {} of {})",
                    current_page_count, sentiment_value, page, total_pages
                )
            } else {
                format!(
                    "Retrieved {} articles with all predictions from database (page {} of {})",
                    current_page_count, page, total_pages
                )
            };
            info!("{}", log_message);

            Ok(PaginatedArticles {
                articles: facet_result.data,
                total_count,
                current_page_count,
                page,
                per_page: limit_count,
                total_pages,
            })
        } else {
            Ok(PaginatedArticles {
                articles: vec![],
                total_count: 0,
                current_page_count: 0,
                page: 1,
                per_page: limit_count,
                total_pages: 0,
            })
        }
    }

    // Keep the old method for backward compatibility
    pub async fn list_articles_with_sentiment(
        &self,
        limit: Option<i64>,
        skip: Option<u64>,
        sentiment: Option<&str>,
    ) -> Result<PaginatedArticles, mongodb::error::Error> {
        // For now, just delegate to the new method
        self.list_articles_with_all_predictions(limit, skip, sentiment)
            .await
    }
}
