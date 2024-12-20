use actix_web::{get, post, web, HttpResponse, Responder};
use chrono::Utc;
use halo2_base::halo2_proofs::halo2curves::bn256::Fr;
use serde_json::json;
use validator::Validate;

use crate::app::{
    dtos::{
        aggregator_request_dto::ProofFromAggregator,
        proposal_dto::{ CreateProposalDto, ProposalByIdResponseDto, UserProofDto},
    },
    entities::{dao_entity::Dao, proposal_entity::{Proposal, UserProof}},
    repository::generic_repository::Repository,
    services::proposal_service::{
            create_proposal, get_all_proposals, get_proposal_by_dao_id,
            get_proposal_by_id, get_result_on_proposal, submit_proof_to_proposal,
        },
};

/// Create a new Proposal
/// 
/// This endpoint creates a new proposal for a Community.
/// 
/// # API Endpoint
/// 
/// ```not_rust
/// POST /proposal
/// Content-Type: application/json
/// ```
/// 
/// # Request Body
/// 
/// The request must include a JSON body with the following fields:
/// 
/// ```json
/// {
///  "creator": "0x11f2b30c9479ccaa639962e943ca7cfd3498705258ddb49dfe25bba00a555e48cb35a79f3d084ce26dbac0e6bb887463774817cb80e89b20c0990bc47f9075d5",
///  "title": "Proposal 3",
///  "description": "This proposal aims to improve our current infrastructure by adopting new technologies and methodologies.",
///  "dao_id": "6614077226af72332791da5f",
///  "end_time": "2025-10-12T07:14:44.077Z",
///  "start_time": "2025-10-12T07:09:37.233Z",
///  "voting_options": ["yes", "no"],
///  "membership_root": "0x1f38b57f3bdf96f05ea62fa68814871bf0ca8ce4dbe073d8497d5a6b0a53e5e0",
///  "nullifier": "0x0339861e70a9bdb6b01a88c7534a3332db915d3d06511b79a5724221a6958fbe",
///  "membership_proof": "0x0339861e70a9bdb6b01a88c7534a3332db915d3d06511b79a5724221a6958fbe"
///  }
/// ```
/// 
/// # Validation Rules
/// Are defined on the `CreateProposalDto` struct
/// 
/// # Response
/// 
/// ## Success (201 Created)
/// 
/// ```json
/// {
///     "message": "Creating proposal",
///    "ObjectId": "507f1f77bcf86cd799439011"
/// }
/// ```
/// 
/// ## Error Responses
/// 
/// ### 400 Bad Request
/// All Errors are defined according to the validation rules
/// 
/// # Example Request
/// ```bash
/// curl -X POST http://api.example.com/proposal \
///      -H "Content-Type: application/json" \
///      -d '{
///         "creator": "0x11f2b30c9479ccaa639962e943ca7cfd3498705258ddb49dfe25bba00a555e48cb35a79f3d084ce26dbac0e6bb887463774817cb80e89b20c0990bc47f9075d5",
///         "title": "Proposal 3",
///         "description": "This proposal aims to improve our current infrastructure by adopting new technologies and methodologies.",
///         "dao_id": "6614077226af72332791da5f",
///         "end_time": "2025-10-12T07:14:44.077Z",
///         "start_time": "2025-10-12T07:09:37.233Z",
///         "voting_options": ["yes", "no"],
///         "membership_root": "0x1f38b57f3bdf96f05ea62fa68814871bf0ca8ce4dbe073d8497d5a6b0a53e5e0",
///         "nullifier": "0x0339861e70a9bdb6b01a88c7534a3332db915d3d06511b79a5724221a6958fbe",
///         "membership_proof": "0x0339861e70a9bdb6b01a88c7534a3332db915d3d06511b79a5724221a6958fbe"
///       }'
/// ```

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

/// Vote on a Proposal
/// 
/// # API Endpoint
/// 
/// ```not_rust
/// POST /proposal/vote/{proposal_id}
/// Content-Type: application/json
/// ```
/// 
/// # Path Parameters
/// 
/// The request must include the proposal_id in the path.
/// 
/// # Request Body
/// 
/// The request must include a JSON body with the following fields:
/// 
/// ```json
/// {
///    "instances": ["0x1", "0x2", "0x3"],
///    "proof": [1, 2, 3]
/// }
/// ```
/// 
/// # Response
/// 
/// ## Success (200 OK)
/// 
/// ```json
/// {
///    "message": "Voting on proposal",
/// }
/// ```
/// 
/// ## Error Responses
/// 
/// ### 400 Bad Request
/// 
/// All Error Messages are defined according to the validation rules

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
    
    let mut proposal = match proposal_db.find_by_id(&proposal_id).await {
        Ok(proposal) => match proposal {
            Some(proposal) => proposal,
            None => {
                return HttpResponse::BadRequest().json(json!({
                    "message": "Failed to find proposal",
                    "Error": "Proposal not found"
                }));
            }
        },
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
                "message": "Failed to find proposal",
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

/// Get Results of a Proposal
/// 
/// # API Endpoint
/// 
/// ```not_rust
/// GET /proposal/result/{proposal_id}
/// ```
/// 
/// # Path Parameters
/// 
/// The request must include the proposal_id in the path.
/// 
/// # Response
/// 
/// ## Success (200 OK)
/// 
/// Returns the result of the proposal voting

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

/// Get a Proposal by its ID
/// 
/// This endpoint retrieves a proposal by its ID.
/// 
/// # API Endpoint
/// 
/// ```not_rust
/// GET /proposal/id/{proposal_id}
/// ```
/// 
/// # Path Parameters
/// 
/// The request must include the proposal_id in the path.
/// 
/// # Response 
/// 
/// ## Success (200 OK)
/// 
/// ```json
/// {
///     "dao_name": "MerkleTree",
///     "dao_id": "6614077226af72332791da5f",
///     "creator_address": "",
///     "proposal_id": "",
///     "proposal_name": "Proposal",
///     "proposal_status": "Inactive",
///     "proposal_description": "This proposal aims to improve our current infrastructure by adopting new technologies and methodologies.",
///     "start_time": "2025-10-12T07:09:37.233Z",
///     "end_time": "2025-10-12T07:14:44.077Z",
///     "encrypted_keys": ""
/// }
/// ```
/// 
/// ## Error Responses
/// 
/// ### 400 Bad Request
/// 
/// Returned when fetching fails:
/// 
/// ```json
/// {
///    "message": "Failed to find proposal",
///    "Error": "Proposal not found"
/// }
/// ```
/// 
/// # Example Request
/// 
/// ```bash
/// curl -X GET http://api.example.com/proposal/id/{proposal_id}
/// ```


#[get("/proposal/id/{proposal_id}")]
async fn get_proposal_by_uid(
    proposal_db: web::Data<Repository<Proposal>>,
    path: web::Path<String>,
) -> impl Responder {
    let proposal_id = path.into_inner();
    match get_proposal_by_id(proposal_db, &proposal_id).await {
        Ok(proposal) =>  HttpResponse::Ok().json(proposal),
        Err(e) => return HttpResponse::BadRequest().json(json!({ "message": "Failed to find proposal", "error": e.to_string() })),
    }
}


// #[post("/proposal/aggregate")]
// async fn submit_aggregated_snark(
//     proposal_db: web::Data<Repository<Proposal>>,
//     res: web::Json<ProofFromAggregator>,
// ) -> impl Responder {
//     let res = res.into_inner();
//     let snark = res.clone().proof;
//     let len = snark.instances[0].len();
//     let proposal_id = u16_from_fr(snark.instances[0][len - 2]);

//     match submit_proof_to_proposal(proposal_db, proposal_id, res).await {
//         Ok(_) => {
//             log::info!("Proof submitted to proposal");
//             return HttpResponse::Ok().json(json!({
//                 "message": "Submitting proof to proposal",
//             }));
//         }
//         Err(e) => {
//             return HttpResponse::BadRequest().json(json!({
//                 "message": "Failed to submit proof to proposal",
//                 "Error": e.to_string()
//             }));
//         }
//     }
// }

// #[get("/proposal/aggregate/{proposal_id}")]
// async fn get_proposal(
//     proposal_db: web::Data<Repository<Proposal>>,
//     path: web::Path<u16>,
// ) -> impl Responder {
//     let proposal_id = path.into_inner();
//     let proposal_id_bson = bson::Bson::Int32(proposal_id as i32); // Convert proposal_id to Bson type
//     match proposal_db
//         .find_by_field("proposalId", proposal_id_bson)
//         .await
//     {
//         // Pass proposal_id_bson to find_by_field
//         Ok(result) => {
//             return HttpResponse::Ok().json(result);
//         }
//         Err(e) => {
//             return HttpResponse::BadRequest().json(json!({
//                 "message": "Failed to get proposal",
//                 "Error": e.to_string()
//             }));
//         }
//     }
// }

/// Get all Proposals
/// 
/// This endpoint retrieves all proposals.
/// 
/// # API Endpoint
/// 
/// ```not_rust
/// GET /proposal/all_proposals
/// ```
/// 
/// # Response Body 
/// 
/// The request must include a JSON body with the following fields:
/// 
/// ```json
/// {
///    "dao_name": "MerkleTree",
///    "dao_id": "6614077226af72332791da5f",
///    "creator_address": "",
///    "proposal_id": "",
///    "proposal_name": "Proposal",
///    "proposal_status": "Inactive",
///    "proposal_description": "This proposal aims to improve our current infrastructure by adopting new technologies and methodologies.",
///    "start_time": "2025-10-12T07:09:37.233Z",
///    "end_time": "2025-10-12T07:14:44.077Z",
/// }
/// ```
/// 
/// ## Error Responses
/// 
/// ### 400 Bad Request
/// All Errors are defined according to the validation rules
/// 
/// # Example Request
/// ```bash
/// curl -X GET http://api.example.com/proposal/all_proposals
/// ```

#[get("/proposal/all_proposals")]
async fn get_proposals(
    db: web::Data<Repository<Proposal>>,
) -> impl Responder {
    match get_all_proposals(db).await {
        Ok(proposals) => {
            return HttpResponse::Ok().json(proposals);
        }
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
                "message": "Failed to get all proposals",
                "Error": e.to_string()
            }));
        }
    }
}

/// Get all Proposals by DAO
/// 
/// # API Endpoint
/// 
/// ```not_rust
/// GET /proposal/proposals_all_by_dao/{dao_id}
/// ```
/// 
/// # Path Parameters
/// 
/// The request must include the dao_id in the path.
/// 
/// # Response
/// 
/// ## Success (200 OK)
/// 
/// ```json
/// {
///   "dao_name": "MerkleTree",
///   "dao_id": "6614077226af72332791da5f",
///   "creator_address": "",
///   "proposal_id": "",
///   "proposal_name": "Proposal",
///   "proposal_status": "Inactive",
///   "proposal_description": "This proposal aims to improve our current infrastructure by adopting new technologies and methodologies.",
///   "start_time": "2025-10-12T07:09:37.233Z",
///   "end_time": "2025-10-12T07:14:44.077Z",
/// }
/// ```
/// 
/// ## Error Responses
/// Will not return error if the DAO does not have any proposals, ONly an empty array will be returned
/// 
/// or 
/// 
/// ```json
/// {
///     "message": "Failed to get all proposals by dao"
///     "Error": ERROR MESSAGE
/// }
/// 
/// # Example Request 
/// 
/// ```bash
/// curl -X GET http://api.example.com/proposals_all_by_dao/{dao_id}
/// ```

#[get("/proposals_all_by_dao/{dao_id}")]
async fn get_all_proposals_by_dao(
    db: web::Data<Repository<Proposal>>,
    path: web::Path<String>,
) -> impl Responder {
    let dao_id = path.into_inner();
    match get_proposal_by_dao_id(db, &dao_id).await {
        Ok(result) => {
            return HttpResponse::Ok().json(result);
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
