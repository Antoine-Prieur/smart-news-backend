use std::env;

#[derive(Clone)]
pub struct Config {
    pub mongodb_connection_string: String,
    pub mongodb_database_name: String,
    pub articles_collection_name: String,
    pub article_predictions_collection_name: String,
    pub deployment_collection_name: String,
    pub metrics_collection_name: String,
    pub predictor_collection_name: String,
}

impl Config {
    pub fn new() -> Result<Self, env::VarError> {
        Ok(Self {
            mongodb_connection_string: env::var("MONGO_URL")
                .unwrap_or_else(|_| "mongodb://admin:password123@localhost:27017".to_string()),
            mongodb_database_name: env::var("MONGODB_DATABASE_NAME")
                .unwrap_or_else(|_| "news".to_string()),

            articles_collection_name: env::var("ARTICLES_COLLECTION_NAME")
                .unwrap_or_else(|_| "articles".to_string()),
            article_predictions_collection_name: env::var("ARTICLE_PREDICTIONS_COLLECTION_NAME")
                .unwrap_or_else(|_| "article_predictions".to_string()),
            deployment_collection_name: env::var("DEPLOYMENT_COLLECTION_NAME")
                .unwrap_or_else(|_| "deployments".to_string()),
            metrics_collection_name: env::var("METRICS_COLLECTION_NAME")
                .unwrap_or_else(|_| "metrics".to_string()),
            predictor_collection_name: env::var("PREDICTOR_COLLECTION_NAME")
                .unwrap_or_else(|_| "predictors".to_string()),
        })
    }
}
