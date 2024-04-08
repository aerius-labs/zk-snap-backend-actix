use crate::app::repository::traits::RepositoryError;
use bson::Bson;
use futures::stream::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Collection,
};
use serde::{Deserialize, Serialize};

use super::traits::RepositoryResult;

pub struct Repository<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Unpin + Sync + Send,
{
    collection: Collection<T>,
}

impl<T> Repository<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Unpin + Sync + Send,
{
    pub fn new(collection: Collection<T>) -> Self {
        Repository { collection }
    }

    pub async fn create(&self, document: T) -> RepositoryResult<String> {
        let result = self
            .collection
            .insert_one(document, None)
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        result
            .inserted_id
            .as_object_id()
            .map(|id| id.to_hex())
            .ok_or(RepositoryError::InternalError(
                "Error parsing ID".to_string(),
            ))
    }

    pub async fn find_all(&self) -> RepositoryResult<Vec<T>> {
        let mut cursor = self
            .collection
            .find(None, None)
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        let mut documents = Vec::new();
        while let Some(doc) = cursor.next().await {
            documents.push(doc.map_err(|e| RepositoryError::InternalError(e.to_string()))?);
        }

        Ok(documents)
    }

    pub async fn find_by_id(&self, id: &str) -> RepositoryResult<Option<T>> {
        let obj_id =
            ObjectId::parse_str(id).map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        let filter = doc! { "_id": obj_id };

        self.collection
            .find_one(filter, None)
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))
    }

    pub async fn if_field_exists(&self, field: &str, value: &str) -> RepositoryResult<bool> {
        let filter = doc! { field: value };
        self.collection
            .find_one(filter, None)
            .await
            .map(|doc| doc.is_some())
            .map_err(|e| RepositoryError::InternalError(e.to_string()))
    }

    pub async fn find_by_field(&self, field: &str, value: Bson) -> RepositoryResult<Option<T>> {
        let filter = doc! { field: value };
        self.collection
            .find_one(filter, None)
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))
    }

    pub async fn find_all_by_field(&self, field: &str, value: Bson) -> RepositoryResult<Vec<T>> {
        let filter = doc! { field: value };
        let mut cursor = self
            .collection
            .find(filter, None)
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        let mut results = Vec::new();
        while let Some(result) = cursor.next().await {
            match result {
                Ok(document) => results.push(document),
                Err(e) => return Err(RepositoryError::InternalError(e.to_string())),
            }
        }

        Ok(results)
    }

    pub async fn update(&self, id: &str, document: T) -> RepositoryResult<()> {
        let obj_id =
            ObjectId::parse_str(id).map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        let filter = doc! { "_id": obj_id };
        let result = self
            .collection
            .replace_one(filter, document, None)
            .await
            .map_err(|e| {
                RepositoryError::InternalError(
                    "Id not found in DAO's collection: ".to_string() + &e.to_string(),
                )
            })?;

        if result.modified_count == 0 {
            Err(RepositoryError::NotFound)
        } else {
            Ok(())
        }
    }

    pub async fn delete(&self, id: &str) -> RepositoryResult<()> {
        let obj_id =
            ObjectId::parse_str(id).map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        let filter = doc! { "_id": obj_id };
        let result = self
            .collection
            .delete_one(filter, None)
            .await
            .map_err(|e| {
                RepositoryError::InternalError("Error deleting DAO: ".to_string() + &e.to_string())
            })?;

        if result.deleted_count == 0 {
            Err(RepositoryError::NotFound)
        } else {
            Ok(())
        }
    }
}
