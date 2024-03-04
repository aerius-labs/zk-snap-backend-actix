use std::io::{Error, ErrorKind};

use crate::app::dtos::dao_dto::CreateDaoDto;
use crate::app::entities::dao_entity::Dao;
use crate::app::repository::repository::Repository;
use actix_web::web;
use mongodb::bson::oid::ObjectId;

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

pub async fn get_all_daos(db: web::Data<Repository<Dao>>) -> Result<Vec<Dao>, Error> {
    match db.find_all().await {
        Ok(result) => Ok(result),
        Err(e) => {
            return Err(Error::new(ErrorKind::Other, e.to_string()));
        }
    }
}
