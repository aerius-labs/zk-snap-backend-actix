use crate::app::dtos::VoterDto;
use actix_web::{post, web, HttpResponse, Responder};
use serde_json::json;

// ! Only for testing
// TODO: Remove after testing
#[post("vote/")]
async fn generate_vote_proof(dto: web::Json<VoterDto>) -> impl Responder {
    let input = dto.into_inner();

    let result = match super::voter_service::generate_voter_proof(input).await {
        Ok(result) => result,
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
                "message": "Failed to generate vote proof",
                "Error": e.to_string()
            }));
        }
    };

    HttpResponse::Ok().json(result)
}