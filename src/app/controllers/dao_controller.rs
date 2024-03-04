use crate::app::repository::repository::Repository;
use crate::app::services::dao_service::{create_dao, get_all_daos};
use crate::app::{dtos::dao_dto::CreateDaoDto, entities::dao_entity::Dao};
use actix_web::{get, post, web, HttpResponse, Responder};
use serde_json::json;

#[post("dao/")]
async fn create(db: web::Data<Repository<Dao>>, dao: web::Json<CreateDaoDto>) -> impl Responder {
    let dao = dao.into_inner();

    let result = match create_dao(db, dao).await {
        Ok(result) => result,
        Err(e) => {
            println!("Failed to create DAO: {}", e);
            return HttpResponse::BadRequest().json(json!({
                "message": "Failed to create DAO",
                "Error": e.to_string()
            }));
        }
    };

    HttpResponse::Created().json(json!({
        "message": "Creating DAO",
        "ObjectId": result
    }))
}

#[get("dao/")]
async fn find_all_daos(db: web::Data<Repository<Dao>>) -> impl Responder {
    let daos = get_all_daos(db).await;
    match daos {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => HttpResponse::BadRequest().json(json!({
          "message": "Failed to get all DAOs",
          "Error": e.to_string()
        })),
    }
}
