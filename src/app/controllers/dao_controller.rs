use actix_web::{get, post, web, Responder};
use mongodb::bson::oid::ObjectId;
use crate::app::{dtos::dao_dto::CreateDaoDto, entities::dao_entity::Dao};
use serde_json::json;
use crate::app::repository::repository::Repository;
use crate::app::services::dao_service::create_dao;

#[get("dao/")]
async fn hello() -> impl Responder {
    web::Json(json!({
        "message": "Hello, DAO!"
    }))
}

#[post("dao/")]
async fn create(
    db: web::Data<Repository<Dao>>,
    dao: web::Json<CreateDaoDto>,
) -> impl Responder {
    let dao = dao.into_inner();
    
    let result = match create_dao(db, dao).await {
        Ok(result) => result,
        Err(e) => {
            println!("Failed to create DAO: {}", e);
            return web::Json(json!({
                "code": 400,
                "message": "Failed to create DAO",
                "Error": e.to_string()
            }));
        }
    };

    web::Json(json!({
        "code": 201,
        "message": "Creating DAO",
        "ObjectId": result
    }))
}
