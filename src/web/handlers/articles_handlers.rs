use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    database::ArticleDocument,
    services::article_service::{ArticleService, SentimentAnalysis},
};

#[derive(Deserialize)]
pub struct ArticlesQuery {
    pub limit: Option<i64>,
    pub skip: Option<u64>,
    pub published_at: Option<String>,
}

#[derive(Serialize)]
pub struct ArticleWithSentimentResponse {
    #[serde(flatten)]
    pub article: ArticleDocument,

    pub sentiment_analysis: Option<SentimentAnalysisResponse>,
}

#[derive(Serialize)]
pub struct SentimentAnalysisResponse {
    pub confidence: Option<f64>,
    pub sentiment: String,
}

#[derive(Serialize)]
pub struct PaginatedArticlesWithSentimentResponse {
    pub articles: Vec<ArticleWithSentimentResponse>,
    pub total_count: u64,
    pub current_page_count: usize,
    pub page: u64,
    pub per_page: i64,
    pub total_pages: u64,
}

impl From<SentimentAnalysis> for SentimentAnalysisResponse {
    fn from(sentiment: SentimentAnalysis) -> Self {
        Self {
            confidence: sentiment.confidence,
            sentiment: sentiment.sentiment,
        }
    }
}

pub async fn get_articles(
    Query(params): Query<ArticlesQuery>,
    State(service): State<ArticleService>,
) -> Result<Json<PaginatedArticlesWithSentimentResponse>, StatusCode> {
    let published_at = params.published_at.as_deref();

    match service
        .get_articles_with_sentiment(params.limit, params.skip, published_at)
        .await
    {
        Ok(paginated_articles) => {
            let articles_response: Vec<ArticleWithSentimentResponse> = paginated_articles
                .articles
                .into_iter()
                .map(|article_with_sentiment| ArticleWithSentimentResponse {
                    article: article_with_sentiment.article,
                    sentiment_analysis: article_with_sentiment.sentiment_analysis.map(|s| s.into()),
                })
                .collect();

            let response = PaginatedArticlesWithSentimentResponse {
                articles: articles_response,
                total_count: paginated_articles.total_count,
                current_page_count: paginated_articles.current_page_count,
                page: paginated_articles.page,
                per_page: paginated_articles.per_page,
                total_pages: paginated_articles.total_pages,
            };

            Ok(Json(response))
        }
        Err(e) => {
            log::error!("Service error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
