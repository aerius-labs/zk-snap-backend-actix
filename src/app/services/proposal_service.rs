use aggregator::wrapper::common::Snark;
use chrono::{DateTime, Utc};
use dotenv::dotenv;
use halo2_base::halo2_proofs::halo2curves::bn256::Fr;
use mongodb::bson::oid::ObjectId;
use pse_poseidon::Poseidon;
use reqwest::Client;
use std::env;
use std::io::{Error, ErrorKind};
use tokio::time::{sleep_until, Instant};
use voter::merkletree::native::MerkleTree;

use super::dao_service;
use crate::app::dtos::aggregator_request_dto::{self, AggregatorBaseDto};
use crate::app::dtos::proposal_dto::MerkleProofVoter;
use crate::app::entities::proposal_entity::EncryptedKeys;
use crate::app::utils::merkle_tree_helper::public_key_to_coordinates;
use crate::app::utils::nullifier_helper::generate_nullifier_root;
use crate::app::utils::parse_string_pub_key::convert_to_public_key_big_int;
use crate::app::{
    dtos::proposal_dto::{CreateProposalDto, DecryptRequest, DecryptResponse},
    entities::{dao_entity::Dao, proposal_entity::Proposal},
    repository::generic_repository::Repository,
    utils::merkle_tree_helper::{from_members_to_leaf, preimage_to_leaf},
};
use actix_web::web;
use num_bigint::BigUint;
use num_traits::Num;

pub async fn create_proposal(
    db: web::Data<Repository<Proposal>>,
    dao_client: web::Data<Repository<Dao>>,
    proposal: CreateProposalDto,
) -> Result<String, Error> {
    let dao = dao_service::get_dao_by_id(dao_client, &proposal.dao_id).await?;

    if !dao.members.contains(&proposal.creator) {
        return Err(Error::new(
            ErrorKind::PermissionDenied,
            "Proposer is not a member of the DAO",
        ));
    }

    match validate_proposal_times(proposal.start_time, proposal.end_time) {
        Ok(_) => (),
        Err(e) => return Err(e),
    };

    // this generates the encrypted keys
    let encrypted_keys = generate_encrypted_keys(proposal.end_time).await?;

    // TODO: Remove this hardcoded value, use root calculation function here for this provided members
    let members_count = dao.members.len();
    let nulifier_root = generate_nullifier_root(members_count as u64)?;

    // this converts the public key to a big int
    let public_key = convert_to_public_key_big_int(&encrypted_keys.pub_key)?;

    // this creates the base proof dto
    let aggregator_request_dto = AggregatorBaseDto {
        pk_enc: public_key,
        membership_root: dao.members_root,
        proposal_id: 0 as u16,
        init_nullifier_root: nulifier_root,
    };

    // this creates the base proof
    let base_proof = create_base_proof(aggregator_request_dto).await?;

    let proposal = Proposal {
        creator: proposal.creator,
        title: proposal.title,
        description: proposal.description,
        dao_id: proposal.dao_id,
        start_time: proposal.start_time,
        end_time: proposal.end_time,
        encrypted_keys: encrypted_keys.clone(),
        voting_options: proposal.voting_options,
        status: "Pending".to_string(),
        result: vec![],
        snark_proof: base_proof,
        id: Some(ObjectId::new()),
    };
    // this schedules the event to handle the end of the proposal
    // let proposal_id = proposal.id.unwrap().to_string();
    // schedule_event(&proposal_id, db.clone(), proposal.end_time).await;

    match db.create(proposal).await {
        Ok(result) => Ok(result),
        Err(e) => Err(Error::new(ErrorKind::Other, e.to_string())),
    }
}

pub async fn get_merkle_proof(
    doa_db: web::Data<Repository<Dao>>,
    dao_id: &str,
    voter_pub_key: &str,
) -> Result<MerkleProofVoter, Error> {
    let dao = dao_service::get_dao_by_id(doa_db, dao_id).await?;
    let members = dao.members;
    let leaves = from_members_to_leaf(&members)?;
    let mut hasher = Poseidon::<Fr, 3, 2>::new(8, 57);
    let merkle_tree = match MerkleTree::new(&mut hasher, leaves) {
        Ok(tree) => tree,
        Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
    };
    let cord: ([Fr; 3], [Fr; 3]) = public_key_to_coordinates(voter_pub_key)?;
    let leaf = preimage_to_leaf(cord);
    let proof = merkle_tree.get_leaf_proof(&leaf);
    let proof = MerkleProofVoter::new(proof.0, proof.1);
    Ok(proof)
}

pub async fn get_proposal_by_id(
    db: web::Data<Repository<Proposal>>,
    id: &str,
) -> Result<Proposal, Error> {
    let proposal = db.find_by_id(id).await.unwrap();
    match proposal {
        Some(proposal) => Ok(proposal),
        None => Err(Error::new(ErrorKind::NotFound, "Proposal not found")),
    }
}

async fn schedule_event(
    proposal_id: &str,
    db: web::Data<Repository<Proposal>>,
    end_time: DateTime<Utc>,
) {
    let now = Utc::now();
    if end_time > now {
        let wait_duration = (end_time - now).to_std().unwrap();
        let sleep_time = Instant::now() + wait_duration;
        sleep_until(sleep_time).await;
    }

    match handle_event_end(proposal_id, db).await {
        Ok(_) => (),
        Err(e) => println!("Failed to handle event end: {}", e),
    }
}

async fn handle_event_end(
    proposal_id: &str,
    db: web::Data<Repository<Proposal>>,
) -> Result<(), Error> {
    let proposal = db.find_by_id(proposal_id).await.unwrap();
    let mut proposal = match proposal {
        Some(proposal) => proposal,
        None => return Err(Error::new(ErrorKind::NotFound, "Proposal not found")),
    };

    let encrypted_keys = proposal.encrypted_keys;
    let mut encrypted_keys = encrypted_keys.clone();

    let pvt_key = decrypt_keys(encrypted_keys.pvt_key).await.unwrap();

    encrypted_keys.pvt_key = pvt_key;

    proposal.encrypted_keys = encrypted_keys;

    match db.update(proposal_id, proposal).await {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::new(ErrorKind::Other, e.to_string())),
    }
}

async fn decrypt_keys(pvt: String) -> Result<String, Error> {
    dotenv().ok();
    let url = env::var("DECRYPTION_URL").expect("URL is not set");
    let client = Client::new();

    // Create the request body
    let req_body = DecryptRequest { pvt };

    // Send a POST request
    let response = client.post(url).json(&req_body).send().await;

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                // Parse the JSON response
                let json: DecryptResponse = match resp.json().await {
                    Ok(json) => json,
                    Err(e) => return Err(Error::new(std::io::ErrorKind::Other, e.to_string())),
                };
                Ok(json.value)
            } else {
                Err(Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to decrypt keys",
                ))
            }
        }
        Err(e) => Err(Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to send request: {}", e),
        )),
    }
}

async fn create_base_proof(aggregator_request_dto: AggregatorBaseDto) -> Result<Snark, Error> {
    let url = env::var("AGGREGATOR_URL").expect("URL is not set");
    let client = Client::new();

    // Send a POST request
    let response = match client.post(url).json(&aggregator_request_dto).send().await {
        Ok(response) => response,
        Err(e) => return Err(Error::new(std::io::ErrorKind::Other, e.to_string())),
    };

    if response.status().is_success() {
        let json: Snark = match response.json().await {
            Ok(json) => json,
            Err(e) => return Err(Error::new(std::io::ErrorKind::Other, e.to_string())),
        };
        Ok(json)
    } else {
        Err(Error::new(
            std::io::ErrorKind::Other,
            "Failed to create base proof",
        ))
    }
}

async fn generate_encrypted_keys(end_time: DateTime<Utc>) -> Result<EncryptedKeys, Error> {
    dotenv().ok();
    let url = env::var("ENCRYPTION_URL").expect("URL is not set");
    let formatted_date_time = end_time.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

    let url = url.to_owned() + &formatted_date_time;

    let response = match reqwest::Client::new().get(&url).send().await {
        Ok(response) => response,
        Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
    };

    if response.status().is_success() {
        let json: EncryptedKeys = match response.json().await {
            Ok(json) => json,
            Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
        };
        Ok(json)
    } else {
        Err(Error::new(
            ErrorKind::Other,
            "Failed to generate encrypted keys",
        ))
    }
}

fn validate_proposal_times(
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
) -> Result<(), Error> {
    let current_time = Utc::now();

    // Check if end time is not greater than start time
    if start_time > end_time {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "end_time should be greater than the start_time",
        ));
    }

    // Check if either start or end time is less than or equal to the current time
    if start_time <= current_time || end_time <= current_time {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "start_time and end_time should be greater than the current date and time",
        ));
    }

    Ok(())
}
