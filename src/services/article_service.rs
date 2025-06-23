use crate::database::ArticleRepository;
use crate::database::repositories::models::article_repository_models::ArticleDocument;
use log::{error, info};

#[derive(Debug, Clone)]
pub struct PaginatedArticlesWithSentiment {
    pub articles: Vec<ArticleDocument>,
    pub total_count: u64,
    pub current_page_count: usize,
    pub page: u64,
    pub per_page: i64,
    pub total_pages: u64,
}

#[derive(Clone)]
pub struct ArticleService {
    article_repository: ArticleRepository,
}

impl ArticleService {
    pub fn new(article_repository: ArticleRepository) -> Self {
        info!("Created ArticleService");
        Self { article_repository }
    }

    pub async fn get_articles_with_sentiment(
        &self,
        limit: Option<i64>,
        skip: Option<u64>,
        sentiment: Option<&str>,
    ) -> Result<PaginatedArticlesWithSentiment, Box<dyn std::error::Error>> {
        info!("Getting articles with sentiment analysis");

        let paginated_articles = self
            .article_repository
            .list_articles_with_sentiment(limit, skip, sentiment)
            .await
            .map_err(|e| {
                error!("Failed to get articles with sentiment filter: {}", e);
                Box::new(e) as Box<dyn std::error::Error>
            })?;

        info!(
            "Successfully enriched {} articles with sentiment analysis",
            paginated_articles.articles.len()
        );

        Ok(PaginatedArticlesWithSentiment {
            articles: paginated_articles.articles,
            total_count: paginated_articles.total_count,
            current_page_count: paginated_articles.current_page_count,
            page: paginated_articles.page,
            per_page: paginated_articles.per_page,
            total_pages: paginated_articles.total_pages,
        })
    }
}
