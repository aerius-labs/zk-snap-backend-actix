use actix_web::{get, post, web, HttpResponse, Responder};
use aggregator::wrapper::common::Snark;
use halo2_base::{halo2_proofs::halo2curves::bn256::Fr, utils::biguint_to_fe};
use serde_json::json;
use validator::Validate;
use std::{env, io::Error};

use crate::app::{
    dtos::{dummy_vote_request::VoterDto, proposal_dto::{self, CreateProposalDto}},
    entities::{dao_entity::Dao, proposal_entity::Proposal},
    repository::generic_repository::Repository,
    services::{dao_service::get_dao_by_id, proposal_service::{create_proposal, get_merkle_proof, get_proposal_by_id, submit_vote_to_aggregator}}, utils::parse_string_pub_key::convert_to_public_key_big_int,
};

#[post("proposal/")]
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

#[get("proposal/vote/{proposal_id}/{voter_pub_key}")]
async fn vote_on_proposal(
    proposal_db: web::Data<Repository<Proposal>>,
    doa_db: web::Data<Repository<Dao>>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (proposal_id, voter_pub_key) = path.into_inner();
    let user_proof = match create_vote_dto(proposal_db.clone(), doa_db.clone(), &proposal_id, &voter_pub_key).await {
        Ok(result) => result,
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
                "message": "Failed to create vote dto",
                "Error": e.to_string()
            }));
        }
    };

    let snark = match dummy_vote_call(user_proof).await {
        Ok(result) => result,
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
                "message": "Failed to vote on proposal",
                "Error": e.to_string()
            }));
        }
    };
    
    match submit_vote_to_aggregator(&proposal_id, snark, proposal_db).await {
        Ok(_) => {
            return HttpResponse::Ok().json(json!({
                "message": "Voting on proposal",
            }));
        }
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
                "message": "Failed to vote on proposal",
                "Error": e.to_string()
            }));
        }
    }
}

#[get("proposal/{proposal_id}")]
async fn get_results(
    proposal_db: web::Data<Repository<Proposal>>,
    path: web::Path<String>,
) -> impl Responder {
    let proposal_id = path.into_inner();
    return HttpResponse::Ok().json(json!({
        "message": "Getting results",
        "proposal_id": proposal_id
    }));
}

// TODO: Delete this function once after wasm is done
async fn create_vote_dto(
    proposal_db: web::Data<Repository<Proposal>>,
    doa_db: web::Data<Repository<Dao>>,
    proposal_id: &str,
    voter_pub_key: &str,
) -> Result<VoterDto, Error> {
    let proposal = get_proposal_by_id(proposal_db, proposal_id).await?;
    let dao = get_dao_by_id(doa_db.clone(), &proposal.dao_id).await?;

    let merkle_root = biguint_to_fe::<Fr>(&dao.members_root);
    let membership_proof_and_helper: proposal_dto::MerkleProofVoter = get_merkle_proof(doa_db.clone(), &proposal.dao_id, voter_pub_key).await?;
    let membership_proof = membership_proof_and_helper.proof;
    let helper = membership_proof_and_helper.helper;
    let pk_enc = convert_to_public_key_big_int(&proposal.encrypted_keys.pub_key)?;
    
    let vote_dto = VoterDto {
        proposal_id: 0 as u16,
        pk_enc,
        membership_root: merkle_root,
        membership_proof,
        membership_proof_helper: helper,
    };

    println!("{:?}", vote_dto);

    Ok(vote_dto)
}

//TODO: Delete this function once after wasm is done
async fn dummy_vote_call(vote_dto: VoterDto) -> Result<Snark, Error> {
    let client = reqwest::Client::new();
    let url = env::var("DUMMY_VOTE_URL").expect("URL is not set");

    let response = match client.post(url).json(&vote_dto).send().await {
        Ok(response) => response,
        Err(e) => return Err(Error::new(std::io::ErrorKind::Other, e.to_string())),
    };

    println!("{:?}", response.status());

    if response.status().is_success() {
        let json: Snark = match response.json().await {
            Ok(json) => json,
            Err(e) => return Err(Error::new(std::io::ErrorKind::Other, e.to_string())),
        };
        Ok(json)
    } else {
        Err(Error::new(
            std::io::ErrorKind::Other,
            "Failed to vote on proposal",
        ))
    }
}