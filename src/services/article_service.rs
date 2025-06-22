use crate::database::repositories::models::article_prediction_repository_models::ArticlePredictionsDocument;
use crate::database::repositories::models::article_repository_models::ArticleDocument;
use crate::database::{ArticlePredictionsRepository, ArticleRepository};
use log::{error, info, warn};

#[derive(Debug, Clone)]
pub struct ArticleWithSentiment {
    pub article: ArticleDocument,
    pub sentiment_analysis: Option<SentimentAnalysis>,
}

#[derive(Debug, Clone)]
pub struct SentimentAnalysis {
    pub confidence: Option<f64>,
    pub sentiment: String,
}

#[derive(Debug, Clone)]
pub struct PaginatedArticlesWithSentiment {
    pub articles: Vec<ArticleWithSentiment>,
    pub total_count: u64,
    pub current_page_count: usize,
    pub page: u64,
    pub per_page: i64,
    pub total_pages: u64,
}

#[derive(Clone)]
pub struct ArticleService {
    article_repository: ArticleRepository,
    predictions_repository: ArticlePredictionsRepository,
}

impl ArticleService {
    pub fn new(
        article_repository: ArticleRepository,
        predictions_repository: ArticlePredictionsRepository,
    ) -> Self {
        info!("Created ArticleService");
        Self {
            article_repository,
            predictions_repository,
        }
    }

    pub async fn get_articles_with_sentiment(
        &self,
        limit: Option<i64>,
        skip: Option<u64>,
        published_at: Option<&str>,
    ) -> Result<PaginatedArticlesWithSentiment, Box<dyn std::error::Error>> {
        info!("Getting articles with sentiment analysis");

        let paginated_articles = self
            .article_repository
            .list_articles(limit, skip, published_at)
            .await
            .map_err(|e| {
                error!("Failed to get articles: {}", e);
                Box::new(e) as Box<dyn std::error::Error>
            })?;

        let mut articles_with_sentiment = Vec::new();

        for article in paginated_articles.articles {
            let article_id = match &article.id {
                Some(id) => *id,
                _none => {
                    warn!("Article without ID found, skipping sentiment lookup");
                    articles_with_sentiment.push(ArticleWithSentiment {
                        article,
                        sentiment_analysis: None,
                    });
                    continue;
                }
            };

            let sentiment_analysis = self.get_sentiment_for_article(article_id).await;

            articles_with_sentiment.push(ArticleWithSentiment {
                article,
                sentiment_analysis,
            });
        }

        info!(
            "Successfully enriched {} articles with sentiment analysis",
            articles_with_sentiment.len()
        );

        Ok(PaginatedArticlesWithSentiment {
            articles: articles_with_sentiment,
            total_count: paginated_articles.total_count,
            current_page_count: paginated_articles.current_page_count,
            page: paginated_articles.page,
            per_page: paginated_articles.per_page,
            total_pages: paginated_articles.total_pages,
        })
    }

    async fn get_sentiment_for_article(
        &self,
        article_id: mongodb::bson::oid::ObjectId,
    ) -> Option<SentimentAnalysis> {
        match self
            .predictions_repository
            .find_by_article_id_and_prediction_type(article_id, "sentiment_analysis")
            .await
        {
            Ok(Some(prediction)) => self.extract_sentiment_from_prediction(prediction),
            Ok(_none) => {
                info!("No sentiment analysis found for article {}", article_id);
                None
            }
            Err(e) => {
                error!(
                    "Error getting sentiment analysis for article {}: {}",
                    article_id, e
                );
                None
            }
        }
    }

    fn extract_sentiment_from_prediction(
        &self,
        prediction: ArticlePredictionsDocument,
    ) -> Option<SentimentAnalysis> {
        let selected_prediction_id = prediction.selected_prediction;

        if let Some(selected_prediction) = prediction.predictions.get(&selected_prediction_id) {
            match &selected_prediction.prediction_value {
                serde_json::Value::Object(obj) => {
                    let sentiment = obj
                        .get("sentiment")
                        .or_else(|| obj.get("label"))
                        .or_else(|| obj.get("classification"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();

                    Some(SentimentAnalysis {
                        confidence: selected_prediction.prediction_confidence,
                        sentiment,
                    })
                }
                serde_json::Value::String(sentiment_str) => Some(SentimentAnalysis {
                    confidence: selected_prediction.prediction_confidence,
                    sentiment: sentiment_str.clone(),
                }),
                _ => {
                    warn!(
                        "Unexpected prediction_value format for sentiment analysis: {:?}",
                        selected_prediction.prediction_value
                    );
                    None
                }
            }
        } else {
            error!(
                "Selected prediction ID {} not found in predictions HashMap",
                selected_prediction_id
            );
            None
        }
    }
}
