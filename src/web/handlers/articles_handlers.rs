use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};

use crate::database::{ArticleDocument, ArticleRepository};

#[derive(Deserialize)]
pub struct ArticlesQuery {
    pub limit: Option<i64>,
    pub skip: Option<u64>,
    pub published_at: Option<String>,
}

#[derive(Serialize)]
pub struct PaginatedArticlesResponse {
    pub articles: Vec<ArticleDocument>,
    pub total_count: u64,
    pub current_page_count: usize,
    pub page: u64,
    pub per_page: i64,
    pub total_pages: u64,
}

pub async fn get_articles(
    Query(params): Query<ArticlesQuery>,
    State(repository): State<ArticleRepository>,
) -> Result<Json<PaginatedArticlesResponse>, StatusCode> {
    let published_at = params.published_at.as_deref();

    match repository
        .list_articles(params.limit, params.skip, published_at)
        .await
    {
        Ok(paginated_articles) => {
            let total_count = paginated_articles.total_count;
            let current_page_count = paginated_articles.current_page_count;
            let page = paginated_articles.page;
            let per_page = paginated_articles.per_page;
            let total_pages = paginated_articles.total_pages;
            let response = PaginatedArticlesResponse {
                articles: paginated_articles.articles,
                total_count,
                current_page_count,
                page,
                per_page,
                total_pages,
            };

            Ok(Json(response))
        }
        Err(e) => {
            log::error!("Database error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
