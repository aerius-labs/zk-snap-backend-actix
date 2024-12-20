use std::io::{Error, ErrorKind};

use crate::app::dtos::dao_dto::{CreateDaoDto, DaoResponseDto};
use crate::app::entities::dao_entity::Dao;
use crate::app::repository::generic_repository::Repository;
use actix_web::web;
use mongodb::bson::oid::ObjectId;

pub async fn create_dao(
    db: web::Data<Repository<Dao>>,
    dao: CreateDaoDto,
) -> Result<String, Error> {
    let dao_entity = Dao {
        id: Some(ObjectId::new()),
        name: dao.name,
        description: dao.description,
        logo: dao.logo,
    };

    let object_id = match db.create(dao_entity).await {
        Ok(result) => result,
        Err(e) => {
            return Err(Error::new(ErrorKind::Other, e.to_string()));
        }
    };

    Ok(object_id)
}

/// Returns a list of all DAOs
/// Adds a logo in response if not present in the DAO
pub async fn get_all_daos(db: web::Data<Repository<Dao>>) -> Result<Vec<DaoResponseDto>, Error> {
    match db.find_all_projected().await {
        Ok(result) => {
            let dao_resp: Vec<DaoResponseDto> = result
                .into_iter()
                .map(|dao| DaoResponseDto {
                    name: dao.name,
                    id: dao.id.unwrap().to_string(),
                    logo: dao.logo.unwrap_or_else( ||
                        "https://as1.ftcdn.net/v2/jpg/05/14/25/60/1000_F_514256050_E5sjzOc3RjaPSXaY3TeaqMkOVrXEhDhT.jpg".to_string(),
                    ),
                })
                .collect();
            Ok(dao_resp)
        }
        Err(e) => Err(Error::new(ErrorKind::Other, e.to_string())),
    }
}

/// Returns a DAO by ID
pub async fn get_dao_by_id(db: web::Data<Repository<Dao>>, id: &str) -> Result<DaoResponseDto, Error> {
    let result = db.find_by_id_projected(id).await.map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
    match result {
        Some(dao) => Ok(dao),
        None => Err(Error::new(ErrorKind::NotFound, "DAO not found")),
    }
}

pub async fn delete_by_id(db: web::Data<Repository<Dao>>, id: &str) -> Result<(), Error> {
    match db.delete(id).await {
        Ok(result) => Ok(result),
        Err(e) => Err(Error::new(ErrorKind::Other, e.to_string())),
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
    };

    match db.update(id, dao_entity).await {
        Ok(result) => Ok(result),
        Err(e) => Err(Error::new(ErrorKind::Other, e.to_string())),
    }
}
