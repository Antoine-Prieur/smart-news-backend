use std::env;

#[derive(Clone)]
pub struct Config {
    pub mongodb_connection_string: String,
    pub mongodb_database_name: String,
    pub mongodb_collection_name: String,
}

impl Config {
    pub fn new() -> Result<Self, env::VarError> {
        Ok(Self {
            mongodb_connection_string: env::var("MONGO_URL")
                .unwrap_or_else(|_| "mongodb://admin:password123@localhost:27017".to_string()),
            mongodb_database_name: env::var("MONGODB_DATABASE_NAME")
                .unwrap_or_else(|_| "news".to_string()),
            mongodb_collection_name: env::var("MONGODB_COLLECTION_NAME")
                .unwrap_or_else(|_| "articles".to_string()),
        })
    }
}
