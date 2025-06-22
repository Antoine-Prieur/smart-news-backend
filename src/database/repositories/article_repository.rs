use log::{error, info};
use mongodb::Collection;
use mongodb::bson::doc;

use crate::database::mongo_client::DatabaseClient;

use super::models::article_repository_models::{ArticleDocument, PaginatedArticles};

#[derive(Clone)]
pub struct ArticleRepository {
    collection: Collection<ArticleDocument>,
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
        published_at: Option<&str>,
    ) -> Result<PaginatedArticles, mongodb::error::Error> {
        let mut filter = doc! {};
        if let Some(pub_date) = published_at {
            filter.insert("published_at", pub_date);
        }

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
}
