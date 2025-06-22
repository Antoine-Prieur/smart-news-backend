pub mod mongo_client;
pub mod repositories;

pub use repositories::article_prediction_repository::ArticlePredictionsRepository;
pub use repositories::article_repository::ArticleRepository;
pub use repositories::models::article_repository_models::ArticleDocument;
