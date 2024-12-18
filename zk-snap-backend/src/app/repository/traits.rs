use async_trait::async_trait;
use bson::{oid::ObjectId, Document};
use serde::{Deserialize, Serialize};

// Custom error type that can be extended to wrap various database errors
#[derive(Debug)]
pub enum RepositoryError {
    NotFound,
    InternalError(String),
}

impl std::fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepositoryError::NotFound => write!(f, "Item not found"),
            RepositoryError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for RepositoryError {}

pub type RepositoryResult<T> = Result<T, RepositoryError>;
#[async_trait]
pub trait DataRepository<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Sync + Send + Clone,
{
    async fn create(&self, document: T) -> RepositoryResult<String>; // Returns ID
    async fn find_all(&self) -> RepositoryResult<Vec<T>>;
    async fn find_by_id(&self, id: &str) -> RepositoryResult<Option<T>>;
    async fn update(&self, id: &str, document: T) -> RepositoryResult<()>;
    async fn delete(&self, id: &str) -> RepositoryResult<()>;
}

/// This projectable trait is used to project the fields of a document
pub trait Projectable {
    fn get_projection_pipeline(id: ObjectId) -> Vec<Document>;
}