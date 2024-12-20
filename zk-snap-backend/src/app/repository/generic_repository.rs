use crate::app::{dtos::dao_dto::DaoProjectedFields, repository::traits::RepositoryError};
use bson::Bson;
use futures::stream::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId}, options::{Acknowledgment, AggregateOptions, InsertOneOptions, WriteConcern}, Collection
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::traits::{Projectable, ProjectableByField, RepositoryResult};

/// Generic repository for CRUD operations on a MongoDB collection.
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
        let write_concern = WriteConcern::builder()
            .w(Acknowledgment::Majority)
            .journal(false)
            .build();

        let options = InsertOneOptions::builder()
            .write_concern(write_concern)
            .build();

        let result = self
            .collection
            .insert_one(document, options)
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

    /// Retrieves all DAO records from the database with only specified fields (name, logo, id) using MongoDB aggregation.
    /// Returns a vector of DaoProjectFields DTOs, which contains the minimal required data for the DAO response.
    /// Uses batch processing with a size of 100 for optional performace when handling large datasets.
    pub async fn find_all_projected(&self) -> RepositoryResult<Vec<DaoProjectedFields>> {
        let pipeline = DaoProjectedFields::projection_doc();

        let options = AggregateOptions::builder()
            .batch_size(100)
            .allow_disk_use(true)
            .build();

            let mut cursor = self
            .collection
            .aggregate(pipeline, options)
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        let mut documents = Vec::with_capacity(100);
        while let Some(doc_result) = cursor.next().await {
            match doc_result {
                Ok(doc) => {
                    match bson::from_document(doc) {
                        Ok(dto) => documents.push(dto),
                        Err(e) => {
                            eprintln!("Error deserializing document: {}", e);
                            continue;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error fetching document: {}", e);
                    continue;
                }
            }
        }

        Ok(documents)

    }
    
    /// Reterieves all documents from the database with only specified fields using MongoDB aggregation.
    pub async fn find_all_with_projection<R>(&self) -> RepositoryResult<Vec<R>>
    where R: DeserializeOwned + Projectable
    {

        let pipeline = R::get_projection_pipeline(None);

        let options = mongodb::options::AggregateOptions::builder()
            .batch_size(100)
            .allow_disk_use(true)
            .build();

        let mut cursor = self
            .collection
            .aggregate(pipeline, options)
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        let mut proposals = Vec::with_capacity(100);
        while let Some(doc_result) = cursor.next().await {
            match doc_result {
                Ok(doc) => {
                    match bson::from_document(doc) {
                        Ok(dto) => proposals.push(dto),
                        Err(e) => return Err(RepositoryError::InternalError(e.to_string())),
                    }
                }
                Err(e) => return Err(RepositoryError::InternalError(e.to_string())),
            }
        }

        Ok(proposals)
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

    /// Retrieves a single document from the database with only specified fields using MongoDB aggregation.
    pub async fn find_by_id_projected<R>(&self, id: &str) -> RepositoryResult<Option<R>>
    where R: DeserializeOwned + Projectable
    {
        let obj_id =
            ObjectId::parse_str(id).map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        let pipeline = R::get_projection_pipeline(Some(obj_id));

        let mut cursor = self
            .collection
            .aggregate(pipeline, None)
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        if let Some(doc_result) = cursor.next().await {
            match doc_result {
                Ok(doc) => {
                    match bson::from_document(doc) {
                        Ok(dto) => Ok(Some(dto)),
                        Err(e) => {
                            Err(RepositoryError::InternalError(e.to_string()))
                        },
                    }
                }
                Err(e) => Err(RepositoryError::InternalError(e.to_string())),
            }
        } else {
            Ok(None)
        }
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

    pub async fn find_all_with_projection_by_field<R>(&self, dao_id: &str) -> RepositoryResult<Vec<R>> 
        where R: DeserializeOwned + ProjectableByField
    {
        let pipeline = R::get_projection_pipeline_by_field(dao_id);

        let options = mongodb::options::AggregateOptions::builder()
            .batch_size(100)
            .allow_disk_use(true)
            .build();

        let mut cursor = self
            .collection
            .aggregate(pipeline, options)
            .await
            .map_err(|e| RepositoryError::InternalError(e.to_string()))?;

        let mut proposals = Vec::with_capacity(100);
        while let Some(doc_result) = cursor.next().await {
            match doc_result {
                Ok(doc) => {
                    match bson::from_document(doc) {
                        Ok(dto) => proposals.push(dto),
                        Err(e) => {
                            eprintln!("Error deserializing document: {}", e);
                            continue;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error fetching document: {}", e);
                    continue;
                }
            }
        }

        Ok(proposals)
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
