use actix_web::{get, post, web, Responder};
use mongodb::bson::oid::ObjectId;
use crate::app::{dtos::dao_dto::CreateDaoDto, entities::dao_entity::Dao};
use serde_json::json;
use crate::app::services::dao_service::DaoService;

#[get("dao/")]
async fn hello() -> impl Responder {
    web::Json(json!({
        "message": "Hello, DAO!"
    }))
}

#[post("dao/")]
async fn create(
    db: web::Data<DaoService>,
    dao: web::Json<CreateDaoDto>,
) -> impl Responder {
    let dao = dao.into_inner();
    
    if dao.members.is_empty() {
        return web::Json(json!({
            "message": "Members are required"
        }));
    }

    let dao_entity = Dao {
        id: Some(ObjectId::new()),
        name: dao.name,
        description: dao.description,
        logo: dao.logo,
        members: dao.members,
    };

    let result = match db.create(dao_entity).await {
        Ok(result) => result,
        Err(e) => {
            println!("Failed to create DAO: {}", e);
            return web::Json(json!({
                "message": "Failed to create DAO"
            }));
        }
    };


    web::Json(json!({
        "message": "Creating DAO",
        "ObjectId": result
    }))
}
