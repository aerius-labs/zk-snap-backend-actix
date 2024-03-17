use actix_web::{web, post, HttpResponse, Responder};
use crate::app::dtos::AggregatorBaseDto;

use super::dtos::AggregatorReccursiveDto;

#[post("base/")]
async fn generate_base_proof(wit: web::Json<AggregatorBaseDto>) -> impl Responder {
    HttpResponse::Ok().body("Hello, world!")
}

#[post("reccursive/")]
async fn generate_reccursive_proof(wit: web::Json<AggregatorReccursiveDto>) -> impl Responder {
    HttpResponse::Ok().body("Hello, world!")
}
