use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};

use crate::{database::ArticleDocument, web::routes::AppState};

#[derive(Deserialize)]
pub struct ArticlesQuery {
    pub limit: Option<i64>,
    pub skip: Option<u64>,
    pub sentiment: Option<String>,
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
    State(app_state): State<AppState>,
) -> Result<Json<PaginatedArticlesResponse>, StatusCode> {
    let sentiment = params.sentiment.as_deref();

    match app_state
        .article_service
        .get_articles_with_all_predictions(params.limit, params.skip, sentiment)
        .await
    {
        Ok(paginated_articles) => {
            let response = PaginatedArticlesResponse {
                articles: paginated_articles.articles,
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
