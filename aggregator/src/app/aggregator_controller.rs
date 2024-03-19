use crate::app::dtos::AggregatorBaseDto;
use actix_web::{post, web, HttpResponse, Responder};
use serde_json::json;

use super::dtos::AggregatorRecursiveDto;

#[post("base/")]
async fn generate_base_proof(dto: web::Json<AggregatorBaseDto>) -> impl Responder {
    let input = dto.into_inner();

    let result = match super::aggregator_service::generate_base_proof(input).await {
        Ok(result) => result,
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
                "message": "Failed to generate base proof",
                "Error": e.to_string()
            }));
        }
    };

    HttpResponse::Ok().json(result)
}

#[post("recursive/")]
async fn generate_recursive_proof(dto: web::Json<AggregatorRecursiveDto>) -> impl Responder {
    let input = dto.into_inner();

    let result = match super::aggregator_service::generate_recursive_proof(input).await {
        Ok(result) => result,
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
                "message": "Failed to generate recursive proof",
                "Error": e.to_string()
            }));
        }
    };

    HttpResponse::Ok().json(result)
}
