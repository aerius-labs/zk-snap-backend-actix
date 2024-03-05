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

pub async fn get_dao_by_id(db: web::Data<Repository<Dao>>, id: &String) -> Result<Dao, Error> {
    let obj_id = ObjectId::parse_str(id).unwrap();
    let result = match db.find_by_id(obj_id).await {
        Ok(result) => result,
        Err(e) => {
            return Err(Error::new(ErrorKind::Other, e.to_string()));
        }
    }
    .unwrap();

    Ok(result)
}

pub async fn delete_by_id(db: web::Data<Repository<Dao>>, id: &String) -> Result<(), Error> {
    let obj_id = ObjectId::parse_str(id).unwrap();

    let result = match db.delete(obj_id).await {
        Ok(result) => result,
        Err(e) => {
            return Err(Error::new(ErrorKind::Other, e.to_string()));
        }
    };

    if result.deleted_count == 0 {
        return Err(Error::new(
            ErrorKind::NotFound,
            "Id not found in DAOs collection.",
        ));
    } else {
        Ok(())
    }
}

pub async fn update_dao_by_id(
    db: web::Data<Repository<Dao>>,
    id: &String,
    dao: CreateDaoDto,
) -> Result<(), Error> {
    let obj_id = ObjectId::parse_str(id).unwrap();

    let dao_entity = Dao {
        id: Some(obj_id),
        name: dao.name,
        description: dao.description,
        logo: dao.logo,
        members: dao.members,
    };

    let result = match db.update(obj_id, dao_entity).await {
        Ok(result) => result,
        Err(e) => {
            return Err(Error::new(ErrorKind::Other, e.to_string()));
        }
    };

    if result.modified_count == 0 {
        return Err(Error::new(
            ErrorKind::NotFound,
            "Id not found in DAOs collection.",
        ));
    } else {
        Ok(())
    }
}
