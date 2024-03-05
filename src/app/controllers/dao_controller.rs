use crate::app::repository::repository::Repository;
use crate::app::services::dao_service::{
    create_dao, delete_by_id, get_all_daos, get_dao_by_id, update_dao_by_id,
};
use crate::app::{dtos::dao_dto::CreateDaoDto, entities::dao_entity::Dao};
use actix_web::web::Path;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use serde_json::json;

#[post("dao/")]
async fn create(db: web::Data<Repository<Dao>>, dao: web::Json<CreateDaoDto>) -> impl Responder {
    let dao = dao.into_inner();

    let result = match create_dao(db, dao).await {
        Ok(result) => result,
        Err(e) => {
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

#[get("dao/{id}")]
async fn find_by_id(db: web::Data<Repository<Dao>>, path: Path<String>) -> impl Responder {
    let id = path.into_inner();
    if id.is_empty() {
        return HttpResponse::BadRequest().body("Invalid id");
    }
    let dao = match get_dao_by_id(db, &id).await {
        Ok(result) => result,
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
                "message": "Failed to get DAO by id",
                "Error": e.to_string()
            }));
        }
    };

    HttpResponse::Ok().json(dao)
}

#[delete("dao/{id}")]
async fn delete_dao(db: web::Data<Repository<Dao>>, path: Path<String>) -> impl Responder {
    let id = path.into_inner();
    if id.is_empty() {
        return HttpResponse::BadRequest().body("Invalid id");
    }
    let _ = match delete_by_id(db, &id).await {
        Ok(result) => result,
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
                "message": "Failed to delete DAO",
                "Error": e.to_string()
            }));
        }
    };

    HttpResponse::Ok().json(json!({
        "message": "Deleted DAO",
    }))
}

#[put("dao/{id}")]
async fn update_dao(
    db: web::Data<Repository<Dao>>,
    path: Path<String>,
    dao: web::Json<CreateDaoDto>,
) -> impl Responder {
    let id = path.into_inner();
    if id.is_empty() {
        return HttpResponse::BadRequest().body("Invalid id");
    }

    let _ = match update_dao_by_id(db, &id, dao.into_inner()).await {
        Ok(result) => result,
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
                "message": "Failed to update DAO",
                "Error": e.to_string()
            }));
        }
    };

    HttpResponse::Ok().json(json!({
        "message": "Updated DAO",
    }))
}
