use actix_web::{get, post, web, HttpResponse, Responder};
use chrono::Utc;
use halo2_base::halo2_proofs::halo2curves::bn256::Fr;
use serde_json::json;
use validator::Validate;

use crate::app::{
    dtos::{
        aggregator_request_dto::ProofFromAggregator,
        proposal_dto::{self, CreateProposalDto, ProposalByIdResponseDto, ProposalResponseDto, UserProofDto},
    },
    entities::{dao_entity::Dao, proposal_entity::{Proposal, UserProof}},
    repository::generic_repository::Repository,
    services::proposal_service::{
            create_proposal, get_all_proposals, get_proposal_by_dao_id,
            get_proposal_by_id, get_result_on_proposal, submit_proof_to_proposal,
            // submit_vote_to_aggregator,
        },
    utils::parse_string_pub_key::public_key_to_eth_address,
};

#[post("/proposal")]
async fn create(
    db: web::Data<Repository<Proposal>>,
    dao_client: web::Data<Repository<Dao>>,
    proposal: web::Json<CreateProposalDto>,
) -> impl Responder {
    let proposal = proposal.into_inner();
    if proposal.validate().is_err() {
        return HttpResponse::BadRequest().json(json!({
            "message": "Invalid input",
            "Error": proposal.validate().unwrap_err()
        }));
    }

    let now = Utc::now();
    if proposal.start_time <= now {
        return HttpResponse::BadRequest().json(json!({
            "message": "Start time must be in the future",
        }));
    } else if proposal.end_time <= now {
        return HttpResponse::BadRequest().json(json!({
            "message": "End time must be in the future",
        }));
    }else if proposal.end_time <= proposal.start_time {
        return HttpResponse::BadRequest().json(json!({
            "message": "End time must be after start time",
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

#[post("/proposal/vote/{proposal_id}")]
async fn vote_on_proposal(
    proposal_db: web::Data<Repository<Proposal>>,
    path: web::Path<String>,
    vote: web::Json<UserProofDto>
) -> impl Responder {
    let proposal_id = path.into_inner();
    let vote = vote.into_inner();
    // check if is_aggregator_available is true
    // if true, submit vote to aggregator
    // else, push user proof in a queue
    
    let mut proposal = match get_proposal_by_id(proposal_db.clone(), &proposal_id).await {
        Ok(result) => result,
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
                "message": "Failed to get proposal",
                "Error": e.to_string()
            }));
        }
    };

    // check if voting on proposal started or not
    if Utc::now() < proposal.start_time {
        return HttpResponse::BadRequest().json(json!({
            "message": "Voting on proposal has not started yet",
        }));
    }

    // check if voting on proposal ended or not
    if Utc::now() > proposal.end_time {
        return HttpResponse::BadRequest().json(json!({
            "message": "Voting on proposal has ended",
        }));
    }

    // Store the vote in the proposal
    let user_proof = UserProof::from_dto(vote);
    proposal.user_proof_array.push(user_proof);

    if let Err(e) = proposal_db.update(&proposal_id, proposal).await {
        return HttpResponse::BadRequest().json(json!({
            "message": "Failed to update proposal",
            "Error": e.to_string()
        }));
    }

    HttpResponse::Ok().json(json!({
        "message": "Voting on proposal",
    }))
}

#[get("/proposal/result/{proposal_id}")]
async fn get_results(
    proposal_db: web::Data<Repository<Proposal>>,
    path: web::Path<String>,
) -> impl Responder {
    let proposal_id = path.into_inner();
    match get_result_on_proposal(proposal_db, &proposal_id).await {
        Ok(result) => {
            return HttpResponse::Ok().json(result);
        }
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
                "message": "Failed to get results",
                "Error": e.to_string()
            }));
        }
    }
}

#[get("/proposal/id/{proposal_id}")]
async fn get_proposal_by_uid(
    proposal_db: web::Data<Repository<Proposal>>,
    dao_db: web::Data<Repository<Dao>>,
    path: web::Path<String>,
) -> impl Responder {
    let proposal_id = path.into_inner();
    let proposal = match get_proposal_by_id(proposal_db, &proposal_id).await {
        Ok(proposal) => proposal,
        Err(e) => return HttpResponse::BadRequest().json(json!({ "message": "Failed to find proposal", "error": e.to_string() })),
    };

    let dao = match dao_db.find_by_id(&proposal.dao_id).await {
        Ok(Some(dao)) => dao,
        Ok(None) => return HttpResponse::NotFound().json(json!({ "message": "DAO not found" })),
        Err(e) => return HttpResponse::InternalServerError().json(json!({ "message": "Error fetching DAO", "error": e.to_string() })),
    };

    let creator_address = match public_key_to_eth_address(&proposal.creator) {
        Ok(address) => address,
        Err(_) => return HttpResponse::InternalServerError().json(json!({ "message": "Failed to convert public key to address" })),
    };

    let resp = ProposalByIdResponseDto {
        dao_name: dao.name,
        creator_address,
        proposal_id,
        proposal_status: proposal.status,
        proposal_description: proposal.description,
        proposal_name: proposal.title.clone(), // Assuming this should match the `proposal_name` field
        start_time: proposal.start_time,
        end_time: proposal.end_time,
        encrypted_keys: proposal.encrypted_keys.clone()
    };

    HttpResponse::Ok().json(resp)
}


#[post("/proposal/aggregate")]
async fn submit_aggregated_snark(
    proposal_db: web::Data<Repository<Proposal>>,
    res: web::Json<ProofFromAggregator>,
) -> impl Responder {
    let res = res.into_inner();
    let snark = res.clone().proof;
    let len = snark.instances[0].len();
    let proposal_id = u16_from_fr(snark.instances[0][len - 2]);

    match submit_proof_to_proposal(proposal_db, proposal_id, res).await {
        Ok(_) => {
            log::info!("Proof submitted to proposal");
            return HttpResponse::Ok().json(json!({
                "message": "Submitting proof to proposal",
            }));
        }
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
                "message": "Failed to submit proof to proposal",
                "Error": e.to_string()
            }));
        }
    }
}

#[get("/proposal/aggregate/{proposal_id}")]
async fn get_proposal(
    proposal_db: web::Data<Repository<Proposal>>,
    path: web::Path<u16>,
) -> impl Responder {
    let proposal_id = path.into_inner();
    let proposal_id_bson = bson::Bson::Int32(proposal_id as i32); // Convert proposal_id to Bson type
    match proposal_db
        .find_by_field("proposalId", proposal_id_bson)
        .await
    {
        // Pass proposal_id_bson to find_by_field
        Ok(result) => {
            return HttpResponse::Ok().json(result);
        }
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
                "message": "Failed to get proposal",
                "Error": e.to_string()
            }));
        }
    }
}

#[get("/proposal/all_proposals")]
async fn get_proposals(
    db: web::Data<Repository<Proposal>>,
    dao_db: web::Data<Repository<Dao>>,
) -> impl Responder {
    match get_all_proposals(db).await {
        Ok(proposals) => {
            let mut proposals_res: Vec<ProposalResponseDto> = Vec::new();
            for proposal in proposals {
                if let Ok(dao) = dao_db.find_by_id(&proposal.dao_id).await {
                    let dao = dao.unwrap();
                    let creator = match public_key_to_eth_address(&proposal.creator) {
                        Ok(x) => x,
                        Err(e) => {
                            return HttpResponse::BadRequest().json(json!({
                                "message": "Failed to get all proposals by dao",
                                "Error": e.to_string()
                            }))
                        }
                    };
                    let dto = ProposalResponseDto {
                        proposal_id: proposal.id.unwrap().to_string(),
                        dao_name: dao.name,
                        dao_logo: dao.logo.unwrap_or("https://as1.ftcdn.net/v2/jpg/05/14/25/60/1000_F_514256050_E5sjzOc3RjaPSXaY3TeaqMkOVrXEhDhT.jpg".to_string()), // Unwrap the Option value
                        creator,
                        title: proposal.title,
                        status: proposal.status,
                        start_time: proposal.start_time,
                        end_time: proposal.end_time,
                        encrypted_keys: proposal.encrypted_keys.clone()
                    };
                    proposals_res.push(dto);
                }
            }
            return HttpResponse::Ok().json(proposals_res);
        }
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
                "message": "Failed to get all proposals",
                "Error": e.to_string()
            }));
        }
    }
}

#[get("/proposals_all_by_dao/{dao_id}")]
async fn get_all_proposals_by_dao(
    db: web::Data<Repository<Proposal>>,
    dao_db: web::Data<Repository<Dao>>,
    path: web::Path<String>,
) -> impl Responder {
    let dao_id = path.into_inner();
    match get_proposal_by_dao_id(db, &dao_id).await {
        Ok(result) => {
            let mut proposals_res: Vec<ProposalResponseDto> = Vec::new();
            for proposal in result {
                if let Ok(dao) = dao_db.find_by_id(&proposal.dao_id).await {
                    let dao = dao.unwrap();
                    let creator = match public_key_to_eth_address(&proposal.creator) {
                        Ok(x) => x,
                        Err(e) => {
                            return HttpResponse::BadRequest().json(json!({
                                "message": "Failed to get all proposals by dao",
                                "Error": e.to_string()
                            }))
                        }
                    };
                    let dto = ProposalResponseDto {
                        proposal_id: proposal.id.unwrap().to_string(),
                        dao_name: dao.name,
                        dao_logo: dao.logo.unwrap_or("https://as1.ftcdn.net/v2/jpg/05/14/25/60/1000_F_514256050_E5sjzOc3RjaPSXaY3TeaqMkOVrXEhDhT.jpg".to_string()), // Unwrap the Option value
                        creator,
                        title: proposal.title,
                        status: proposal.status,
                        start_time: proposal.start_time,
                        end_time: proposal.end_time,
                        encrypted_keys: proposal.encrypted_keys.clone()
                    };
                    proposals_res.push(dto);
                }
            }
            return HttpResponse::Ok().json(proposals_res);
        }
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
                "message": "Failed to get all proposals by dao",
                "Error": e.to_string()
            }));
        }
    }
}

// function to convert proposal_if from Fr to u64
fn u16_from_fr(fr: Fr) -> u16 {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&fr.to_bytes()[0..2]);
    u16::from_le_bytes(bytes.try_into().unwrap())
}
