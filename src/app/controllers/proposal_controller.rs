use actix_web::{get, post, web, HttpResponse, Responder};
use serde_json::json;
use validator::Validate;

use crate::app::{
    dtos::proposal_dto::{self, CreateProposalDto},
    entities::{dao_entity::Dao, proposal_entity::Proposal},
    repository::generic_repository::Repository,
    services::proposal_service::{create_proposal, get_merkle_proof},
};

#[post("proposal/}")]
async fn create(
    db: web::Data<Repository<Proposal>>,
    dao_client: web::Data<Repository<Dao>>,
    proposal: web::Json<CreateProposalDto>,
) -> impl Responder {
    let proposal = proposal.into_inner();

    // Validate input
    if proposal.validate().is_err() {
        return HttpResponse::BadRequest().json(json!({
            "message": "Invalid input",
            "Error": proposal.validate().unwrap_err()
        }));
    }

    // Create proposal
    match create_proposal(db, dao_client, proposal).await {
        Ok(result) => {
            return HttpResponse::Created().json(json!({
                "message": "Creating proposal",
                "ObjectId": result
            }));
        }
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
                "message": "Failed to create proposal",
                "Error": e.to_string()
            }));
        }
    }
}

#[get("proposal/{dao_id}/{member_pub_key}")]
async fn get_merkle_proof_from_pub(
    dao_db: web::Data<Repository<Dao>>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (dao_id, member_pub_key) = path.into_inner();

    match get_merkle_proof(dao_db, &dao_id, &member_pub_key).await {
        Ok(result) => return HttpResponse::Ok().json(result),
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
                "message": "Failed to get merkle proof",
                "Error": e.to_string()
            }));
        }
    };

}