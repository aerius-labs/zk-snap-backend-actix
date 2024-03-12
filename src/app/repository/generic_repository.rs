use crate::app::repository::traits::RepositoryError;
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
        let result = match self.collection.insert_one(document, None).await {
            Ok(result) => result,
            Err(e) => {
                return Err(RepositoryError::InternalError(e.to_string()));
            }
        };
        Ok(result.inserted_id.as_object_id().unwrap().to_string())
    }

    pub async fn find_all(&self) -> RepositoryResult<Vec<T>> {
        let mut result = match self.collection.find(None, None).await {
            Ok(result) => result,
            Err(e) => {
                return Err(RepositoryError::InternalError(e.to_string()));
            }
        };
        let mut documents = Vec::new();
        while let Some(document) = result.next().await {
            match document {
                Ok(doc) => documents.push(doc),
                Err(e) => return Err(RepositoryError::InternalError(e.to_string())),
            }
        }
        Ok(documents)
    }

    pub async fn find_by_id(&self, id: &str) -> RepositoryResult<Option<T>> {
        let obj_id = ObjectId::parse_str(id).unwrap();
        let filter = doc! { "_id": obj_id };
        let result = match self.collection.find_one(filter, None).await {
            Ok(result) => result,
            Err(e) => {
                return Err(RepositoryError::InternalError(e.to_string()));
            }
        };
        Ok(result)
    }

    pub async fn update(&self, id: &str, document: T) -> RepositoryResult<()> {
        let obj_id = ObjectId::parse_str(id).unwrap();
        let filter = doc! { "_id": obj_id };
        let result = self.collection
            .replace_one(filter, document, None)
            .await
            .ok()
            .expect("Id not found in DAOs collection.");
        if result.modified_count == 0 {
            return Err(RepositoryError::NotFound);
        } else {
            return Ok(());
        }
    }

    pub async fn delete(&self, id: &str) -> RepositoryResult<()> {
        let obj_id = ObjectId::parse_str(id).unwrap();
        let filter = doc! { "_id": obj_id };
        let result = self.collection
            .delete_one(filter, None)
            .await
            .ok()
            .expect("Error Deleting DAO");
        if result.deleted_count == 0 {
            return Err(RepositoryError::NotFound);
        } else {
            return Ok(());
        }
    }

    // // pub async fn create(&self, document: T) -> MongoResult<ObjectId> {
    // //     let result = self.collection.insert_one(document, None).await?;
    // //     Ok(result.inserted_id.as_object_id().unwrap().to_owned())
    // // }

    // pub async fn find_all(&self) -> MongoResult<Vec<T>> {
    //     let mut result = self.collection.find(None, None).await?;
    //     let mut documents = Vec::new();
    //     while let Some(document) = result.next().await {
    //         documents.push(document?);
    //     }
    //     Ok(documents)
    // }

    // pub async fn find_by_id(&self, id: ObjectId) -> MongoResult<Option<T>> {
    //     let filter = doc! { "_id": id };
    //     let result = self.collection.find_one(filter, None).await?;
    //     Ok(result)
    // }

    // pub async fn update(&self, id: ObjectId, document: T) -> Result<UpdateResult, Error> {
    //     let filter = doc! { "_id": id };
    //     let result = self
    //         .collection
    //         .replace_one(filter, document, None)
    //         .await
    //         .ok()
    //         .expect("Id not found in DAOs collection.");
    //     Ok(result)
    // }

    // pub async fn delete(&self, id: ObjectId) -> Result<DeleteResult, Error> {
    //     let filter = doc! { "_id": id };
    //     let result = self
    //         .collection
    //         .delete_one(filter, None)
    //         .await
    //         .ok()
    //         .expect("Error Deleting DAO");
    //     Ok(result)
    // }
}

// impl<T> DataRepository<T> for Repository<T>
// where
//     T: Serialize + for<'de> Deserialize<'de> + Unpin + Sync + Send + Clone,
// {
//     async fn create(&self, document: T) -> RepositoryResult<String> {
//         let result = match self.collection.insert_one(document, None).await {
//             Ok(result) => result,
//             Err(e) => {
//                 return Err(RepositoryError::InternalError(e.to_string()));
//             }
//         };
//         Ok(result.inserted_id.as_object_id().unwrap().to_string())
//     }

//     async fn find_all(&self) -> RepositoryResult<Vec<T>> {
//         let mut result = match self.collection.find(None, None).await {
//             Ok(result) => result,
//             Err(e) => {
//                 return Err(RepositoryError::InternalError(e.to_string()));
//             }
//         };
//         let mut documents = Vec::new();
//         while let Some(document) = result.next().await {
//             match document {
//                 Ok(doc) => documents.push(doc),
//                 Err(e) => return Err(RepositoryError::InternalError(e.to_string())),
//             }
//         }
//         Ok(documents)
//     }

//     async fn find_by_id(&self, id: &str) -> RepositoryResult<Option<T>> {
//         let obj_id = ObjectId::parse_str(id).unwrap();
//         let filter = doc! { "_id": obj_id };
//         let result = match self.collection.find_one(filter, None).await {
//             Ok(result) => result,
//             Err(e) => {
//                 return Err(RepositoryError::InternalError(e.to_string()));
//             }
//         };
//         Ok(result)
//     }

//     async fn update(&self, id: &str, document: T) -> RepositoryResult<()> {
//         let obj_id = ObjectId::parse_str(id).unwrap();
//         let filter = doc! { "_id": obj_id };
//         self
//             .collection
//             .replace_one(filter, document, None)
//             .await
//             .ok()
//             .expect("Id not found in DAOs collection.");
//         Ok(())
//     }

//     async fn delete(&self, id: &str) -> RepositoryResult<()> {
//         let obj_id = ObjectId::parse_str(id).unwrap();
//         let filter = doc! { "_id": obj_id };
//         self
//             .collection
//             .delete_one(filter, None)
//             .await
//             .ok()
//             .expect("Error Deleting DAO");
//         Ok(())
//     }
// }
