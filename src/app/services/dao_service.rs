use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Collection,
};
use crate::app::entities::dao_entity::Dao;
use mongodb::error::Result as MongoResult;

pub struct DaoService {
    collection: Collection<Dao>,
}

impl DaoService {
    pub fn new(collection: Collection<Dao>) -> Self {
        DaoService { collection }
    }

    pub async fn create(&self, dao: Dao) -> MongoResult<ObjectId> {
        let result = self.collection.insert_one(dao, None).await?;
        Ok(result.inserted_id.as_object_id().unwrap().to_owned())
    }

    pub async fn find_by_id(&self, id: ObjectId) -> MongoResult<Option<Dao>> {
        let filter = doc! { "_id": id };
        let result = self.collection.find_one(filter, None).await?;
        Ok(result)
    }

    pub async fn update(&self, id: ObjectId, dao: Dao) -> MongoResult<()> {
        let filter = doc! { "_id": id };
        let result = self.collection.replace_one(filter, dao, None).await?;
        Ok(())
    }

    pub async fn delete(&self, id: ObjectId) -> MongoResult<()> {
        let filter = doc! { "_id": id };
        let result = self.collection.delete_one(filter, None).await?;
        Ok(())
    }
}