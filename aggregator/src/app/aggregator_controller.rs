use crate::app::dtos::AggregatorBaseDto;
use actix_web::{post, web, HttpResponse, Responder};

use super::dtos::AggregatorReccursiveDto;

#[post("base/")]
async fn generate_base_proof() -> impl Responder {
    println!("Hello, world!");
    HttpResponse::Ok().body("Hello, world!")
}

#[post("recursive/")]
async fn generate_reccursive_proof(wit: web::Json<AggregatorReccursiveDto>) -> impl Responder {
    HttpResponse::Ok().body("Hello, world!")
}
