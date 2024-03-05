use futures::stream::StreamExt;
use mongodb::error::Result as MongoResult;
use mongodb::results::DeleteResult;
use mongodb::{
    bson::{doc, extjson::de::Error, oid::ObjectId},
    Collection,
};
use serde::{Deserialize, Serialize};

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

    pub async fn create(&self, document: T) -> MongoResult<ObjectId> {
        let result = self.collection.insert_one(document, None).await?;
        Ok(result.inserted_id.as_object_id().unwrap().to_owned())
    }

    pub async fn find_all(&self) -> MongoResult<Vec<T>> {
        let mut result = self.collection.find(None, None).await?;
        let mut documents = Vec::new();
        while let Some(document) = result.next().await {
            documents.push(document?);
        }
        Ok(documents)
    }

    pub async fn find_by_id(&self, id: ObjectId) -> MongoResult<Option<T>> {
        let filter = doc! { "_id": id };
        let result = self.collection.find_one(filter, None).await?;
        Ok(result)
    }

    pub async fn update(&self, id: ObjectId, document: T) -> MongoResult<()> {
        let filter = doc! { "_id": id };
        self.collection.replace_one(filter, document, None).await?;
        Ok(())
    }

    pub async fn delete(&self, id: ObjectId) -> Result<DeleteResult, Error> {
        let filter = doc! { "_id": id };
        let result = self
            .collection
            .delete_one(filter, None)
            .await
            .ok()
            .expect("Error Deleting DAO");
        Ok(result)
    }
}
