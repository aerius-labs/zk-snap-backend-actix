use actix_web::{get, post, web, Responder};
use crate::app::dtos::dao_dto::CreateDaoDto;
use serde_json::json;

#[get("dao/")]
async fn hello() -> impl Responder {
    web::Json(json!({
        "message": "Hello, DAO!"
    }))
}

#[post("dao/")]
async fn create(
    dao: web::Json<CreateDaoDto>,
) -> impl Responder {
    let dao = dao.into_inner();
    println!("Creating DAO: {:?}", dao);
    web::Json(json!({
        "message": "Creating DAO",
        "dao": dao
    }))
}
