use aggregator::wrapper::common::Snark;
use chrono::{DateTime, Utc};
use dotenv::dotenv;
use halo2_base::halo2_proofs::halo2curves::bn256::Fr;
use halo2_base::utils::{biguint_to_fe, ScalarField};
use lapin::{options::*, types::FieldTable, BasicProperties, Connection, ConnectionProperties};
use mongodb::bson::oid::ObjectId;
use pse_poseidon::Poseidon;
use reqwest::Client;
use std::env;
use std::io::{Error, ErrorKind};
use tokio::time::{sleep_until, Instant};
use voter::merkletree::native::MerkleTree;

use super::dao_service;
use crate::app::dtos::aggregator_request_dto::{
    AggregatorBaseDto, AggregatorRecursiveDto, MessageType,
};
use crate::app::dtos::proposal_dto::MerkleProofVoter;
use crate::app::entities::proposal_entity::EncryptedKeys;
use crate::app::utils::index_merkle_tree_helper::update_nullifier_tree;
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
use rand::{thread_rng, Rng};

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
    let (nullifier_root, nullifier_preimages) = generate_nullifier_root(members_count as u64)?;

    // this converts the public key to a big int
    let public_key = convert_to_public_key_big_int(&encrypted_keys.pub_key)?;

    //get random id in u16
    let proposal_id = generate_unique_random_id(db.clone()).await?;

    // this creates the base proof dto
    let aggregator_request_dto = AggregatorBaseDto {
        pk_enc: public_key,
        membership_root: dao.members_root,
        proposal_id,
        init_nullifier_root: nullifier_root.clone(),
    };

    println!("{:?}", aggregator_request_dto);
    // this creates the base proof
    match create_base_proof(aggregator_request_dto).await {
        Ok(_) => (),
        Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
    };

    let proposal = Proposal {
        creator: proposal.creator,
        title: proposal.title,
        proposal_id,
        description: proposal.description,
        dao_id: proposal.dao_id,
        start_time: proposal.start_time,
        end_time: proposal.end_time,
        encrypted_keys: encrypted_keys.clone(),
        voting_options: proposal.voting_options,
        status: "Pending".to_string(),
        result: vec![],
        curr_agg_proof: None,
        is_aggregator_available: false,
        curr_nullifier_root: Some(biguint_to_fe(&nullifier_root)),
        curr_nullifier_preimages: nullifier_preimages,
        user_proof_queue: vec![],
        id: Some(ObjectId::new()),
    };

    match db.create(proposal).await {
        Ok(result) => Ok(result),
        Err(e) => Err(Error::new(ErrorKind::Other, e.to_string())),
    }
}

// Function to generate a unique random ID.
async fn generate_unique_random_id(db: web::Data<Repository<Proposal>>) -> Result<u16, Error> {
    let mut rng = thread_rng();

    loop {
        let random_id = rng.gen::<u16>();
        let id_exists = match db
            .if_field_exists("proposalId", &random_id.to_string())
            .await
        {
            Ok(result) => result,
            Err(_) => false,
        };
        if !id_exists {
            return Ok(random_id);
        }
        // If ID exists, the loop continues and generates a new ID.
    }
}

// Function to submit Snark proof from aggregator to the proposal.
pub async fn submit_proof_to_proposal(
    db: web::Data<Repository<Proposal>>,
    proposal_id: u16,
    snark: Snark,
) -> Result<(), Error> {
    let proposal_bson = bson::Bson::Int32(proposal_id as i32);
    let proposal = match db.find_by_field("proposalId", proposal_bson).await {
        Ok(result) => result,
        Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
    };

    let mut proposal = match proposal {
        Some(proposal) => proposal,
        None => return Err(Error::new(ErrorKind::NotFound, "Proposal not found")),
    };

    let mut num_round: u64 = 0;

    if snark.instances[0].last().is_none() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Invalid previous snark proof",
        ));
    } else {
        num_round = snark.instances[0]
            .last()
            .unwrap()
            .clone()
            .to_u64_limbs(1, 63)[0];
    }

    if num_round > 0 {
        proposal.curr_nullifier_root = Some(snark.instances[0][37]);
    }

    proposal.curr_agg_proof = Some(snark);

    // TODO: check the voter queue if empty than do nothing else consume one voter proof and set is_aggregator_available to false
    let user_proof_queue = proposal.user_proof_queue.clone();
    if user_proof_queue.is_empty() {
        proposal.is_aggregator_available = true;
        let id = proposal.id.unwrap().to_string();
        match db.update(&id, proposal).await {
            Ok(_) => {
                println!("Proof submitted to proposal");
                Ok(())
            },
            Err(e) => Err(Error::new(ErrorKind::Other, e.to_string())),
        }
    } else {
        match submit_to_aggregator_from_queue(proposal, db.clone()).await {
            Ok(_) => {
                println!("Proof submitted to proposal from queue");
                Ok(())
            },
            Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
        }
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

async fn submit_to_aggregator_from_queue(
    proposal: Proposal,
    proposal_db: web::Data<Repository<Proposal>>,
) -> Result<(), Error>{
    let mut proposal = proposal;
    let mut user_proof_queue = proposal.user_proof_queue.clone();
    let voter_snark = user_proof_queue.remove(0);
    proposal.user_proof_queue = user_proof_queue;

    let mut hasher = Poseidon::<Fr, 3, 2>::new(8, 57);
    hasher.update(voter_snark.instances[0][24..28].as_ref());
    let nullifier = hasher.squeeze_and_reset();


    let mut num_round: u64 = 0;

    if proposal.curr_agg_proof.clone().unwrap().instances[0].last().is_none() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Invalid previous snark proof",
        ));
    } else {
        num_round = proposal.curr_agg_proof.clone().unwrap().instances[0]
            .last()
            .unwrap()
            .clone()
            .to_u64_limbs(1, 63)[0];
    }
    let nullifier_inputs =
        update_nullifier_tree(proposal.curr_nullifier_preimages, nullifier, num_round + 1);
    let recurr_dto = AggregatorRecursiveDto {
        num_round: num_round as u16,
        voter: voter_snark.clone(),
        previous: proposal.curr_agg_proof.unwrap(),
        nullifier_tree_input: nullifier_inputs.1,
    };

    match call_submit_to_aggregator(recurr_dto).await {
        Ok(_) => println!("recursive proof submited"),
        Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
    }

    proposal.curr_agg_proof = None;
    proposal.curr_nullifier_preimages = nullifier_inputs.0;
    // proposal.curr_nullifier_root = proof.instances[0][37];
    proposal.curr_nullifier_root = None;
    proposal.is_aggregator_available = false;
    let proposal_id = proposal.id.unwrap().to_string();
    match proposal_db.update(&proposal_id, proposal).await {
        Ok(_) => {
            println!("Vote submitted to aggregator from queue");
            Ok(())
        }
        Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
    }
}

pub async fn submit_vote_to_aggregator(
    proposal_id: &str,
    voter_snark: Snark,
    proposal_db: web::Data<Repository<Proposal>>,
) -> Result<(), Error> {
    let proposal = match proposal_db.find_by_id(proposal_id).await {
        Ok(result) => result,
        Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
    };

    let mut proposal = match proposal {
        Some(proposal) => proposal,
        None => return Err(Error::new(ErrorKind::NotFound, "Proposal not found")),
    };

    let previous_snark = match proposal.curr_agg_proof.clone() {
        Some(snark) => snark,
        None => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Invalid previous snark proof",
            ))
        }
    };
    let mut hasher = Poseidon::<Fr, 3, 2>::new(8, 57);
    hasher.update(voter_snark.instances[0][24..28].as_ref());
    let nullifier = hasher.squeeze_and_reset();

    let mut num_round: u64 = 0;

    if previous_snark.instances[0].last().is_none() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Invalid previous snark proof",
        ));
    } else {
        num_round = previous_snark.instances[0]
            .last()
            .unwrap()
            .clone()
            .to_u64_limbs(1, 63)[0];
    }
    let nullifier_inputs =
        update_nullifier_tree(proposal.curr_nullifier_preimages, nullifier, num_round + 1);
    let recurr_dto = AggregatorRecursiveDto {
        num_round: num_round as u16,
        voter: voter_snark.clone(),
        previous: previous_snark,
        nullifier_tree_input: nullifier_inputs.1,
    };

    match call_submit_to_aggregator(recurr_dto).await {
        Ok(_) => println!("recursive proof submited"),
        Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
    }

    proposal.curr_agg_proof = None;
    proposal.curr_nullifier_preimages = nullifier_inputs.0;
    // proposal.curr_nullifier_root = proof.instances[0][37];
    proposal.curr_nullifier_root = None;
    proposal.is_aggregator_available = false;

    match proposal_db.update(proposal_id, proposal).await {
        Ok(_) => {
            println!("Vote submitted to aggregator");
            Ok(())
        }
        Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
    }
}

async fn call_submit_to_aggregator(dto: AggregatorRecursiveDto) -> Result<(), Box<dyn std::error::Error>> {
    // Connect to rabbit MQ
    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());
    let conn = Connection::connect(&addr, ConnectionProperties::default()).await?;
    let channel = conn.create_channel().await?;

    // Declare the queue

    let queue_name = "aggregator_queue";
    channel
        .queue_declare(
            &queue_name,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let msg = MessageType::Recursive(dto);
    channel
        .basic_publish(
            "",
            &queue_name,
            BasicPublishOptions::default(),
            &serde_json::to_vec(&msg)?,
            BasicProperties::default(),
        )
        .await?;

    println!("recursive proof sent to {:?}", &queue_name);
    Ok(())
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

async fn create_base_proof(
    aggregator_request_dto: AggregatorBaseDto,
) -> Result<(), Box<dyn std::error::Error>> {
    // Connect to rabbit MQ
    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());
    let conn = Connection::connect(&addr, ConnectionProperties::default()).await?;
    let channel = conn.create_channel().await?;

    // Declare the queue

    let queue_name = "aggregator_queue";
    channel
        .queue_declare(
            &queue_name,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let msg = MessageType::Base(aggregator_request_dto);
    channel
        .basic_publish(
            "",
            &queue_name,
            BasicPublishOptions::default(),
            &serde_json::to_vec(&msg)?,
            BasicProperties::default(),
        )
        .await?;

    println!("base proof sent to {:?}", &queue_name);
    Ok(())
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
