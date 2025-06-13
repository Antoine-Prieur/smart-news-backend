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
pub struct ArticlesResponse {
    pub articles: Vec<ArticleDocument>,
    pub count: usize,
}

pub async fn get_articles(
    Query(params): Query<ArticlesQuery>,
    State(repository): State<ArticleRepository>,
) -> Result<Json<ArticlesResponse>, StatusCode> {
    let published_at = params.published_at.as_deref();

    match repository
        .list_articles(params.limit, params.skip, published_at)
        .await
    {
        Ok(articles) => {
            let count = articles.len();
            let response = ArticlesResponse { articles, count };

            Ok(Json(response))
        }
        Err(e) => {
            log::error!("Database error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
