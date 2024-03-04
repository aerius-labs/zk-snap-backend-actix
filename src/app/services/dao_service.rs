use std::io::{Error, ErrorKind};

use crate::app::dtos::dao_dto::CreateDaoDto;
use actix_web::web;
use mongodb::bson::oid::ObjectId;
use crate::app::entities::dao_entity::Dao;
use crate::app::repository::repository::Repository;

pub async fn create_dao(
    db: web::Data<Repository<Dao>>,
    dao: CreateDaoDto,
) -> Result<ObjectId, Error> {

    if dao.members.is_empty() {
        return Err(Error::new(ErrorKind::InvalidInput, "Members are required"));
    }

    let dao_entity = Dao {
        id: Some(ObjectId::new()),
        name: dao.name,
        description: dao.description,
        logo: dao.logo,
        members: dao.members,
    };

     let object_id = match db.create(dao_entity).await {
        Ok(result) => result,
        Err(e) => {
            return Err(Error::new(ErrorKind::Other, e.to_string()));
        }
     };

    Ok(object_id)
}