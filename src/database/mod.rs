pub mod models;
pub mod repositories;

pub use models::ArticleDocument;
pub use repositories::article_repository::ArticleRepository;

use async_trait::async_trait;

#[async_trait]
pub trait DatabaseRepository<T> {
    type Error;

    async fn insert_many(&self, items: &[T]) -> Result<(), Self::Error>;
}
