use crate::database::models::ArticleDocument;
use log::{error, info};
use mongodb::bson::doc;
use mongodb::{Client, Collection, Database};

pub struct ArticleRepository {
    collection: Collection<ArticleDocument>,
}

impl ArticleRepository {
    pub async fn new(
        connection_string: &str,
        database_name: &str,
        collection_name: &str,
    ) -> Result<Self, mongodb::error::Error> {
        let client = Client::with_uri_str(connection_string).await?;

        let database: Database = client.database(database_name);

        let collection: Collection<ArticleDocument> = database.collection(collection_name);

        info!(
            "Connected to MongoDB database: {}, collection: {}",
            database_name, collection_name
        );

        Ok(Self { collection })
    }

    pub async fn list_articles(
        &self,
        limit: Option<i64>,
        skip: Option<u64>,
        published_at: Option<&str>,
    ) -> Result<Vec<ArticleDocument>, mongodb::error::Error> {
        let mut filter = doc! {};
        if let Some(pub_date) = published_at {
            filter.insert("published_at", pub_date);
        }

        let mut options = mongodb::options::FindOptions::default();
        if let Some(skip_count) = skip {
            options.skip = Some(skip_count);
        }
        if let Some(limit_count) = limit {
            options.limit = Some(limit_count);
        }

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

        info!("Retrieved {} articles from database", articles.len());

        Ok(articles)
    }
}
