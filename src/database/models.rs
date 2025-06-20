use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

// Articles
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SourceDocument {
    pub id: Option<String>,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ArticleDocument {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub source: SourceDocument,
    pub author: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub url_to_image: Option<String>,
    pub published_at: Option<String>,
    pub content: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct PaginatedArticles {
    pub articles: Vec<ArticleDocument>,
    pub total_count: u64,
    pub current_page_count: usize,
    pub page: u64,
    pub per_page: i64,
    pub total_pages: u64,
}
