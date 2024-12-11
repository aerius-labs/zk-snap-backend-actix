use aggregator::wrapper::common::Snark;
use chrono::{DateTime, Utc};
use dotenv::dotenv;
use halo2_base::halo2_proofs::halo2curves::bn256::Fr;
use halo2_base::utils::{fe_to_biguint, ScalarField};
use lapin::{options::*, types::FieldTable, BasicProperties, Connection, ConnectionProperties};
use mongodb::bson::oid::ObjectId;
use num_bigint::BigUint;
use num_traits::{Num, Zero};
use pse_poseidon::Poseidon;
use paillier_chip::paillier::paillier_add_native;
use reqwest::Client;
use std::env;
use std::io::{Error, ErrorKind};
use tokio::time::{sleep_until, Instant};
use tokio::spawn;
use crate::app::dtos::aggregator_request_dto::{
    AggregatorBaseDto, AggregatorRecursiveDto, MessageType, ProofFromAggregator,
};
use crate::app::dtos::proposal_dto::VoteResultDto;
use crate::app::entities::proposal_entity::{EncryptedKeys, ProposalStatus};
use crate::app::utils::parse_string_pub_key::{convert_to_public_key_big_int, parse_public_key};
use crate::app::{
    dtos::proposal_dto::{CreateProposalDto, DecryptRequest, DecryptResponse},
    entities::{dao_entity::Dao, proposal_entity::Proposal},
    repository::generic_repository::Repository,
};
use actix_web::web;
use rand::{thread_rng, Rng};

fn parse_big_uint(s: &str) -> BigUint {
    let clean_hex = s.trim_start_matches("0x");
    let big_uint = BigUint::from_str_radix(clean_hex, 16).expect("Invalid hex string");
    return big_uint;
}

pub async fn create_proposal(
    db: web::Data<Repository<Proposal>>,
    dao_client: web::Data<Repository<Dao>>,
    proposal: CreateProposalDto,
) -> Result<String, Error> {
    match validate_proposal_times(proposal.start_time, proposal.end_time) {
        Ok(_) => (),
        Err(e) => return Err(e),
    };
    let encrypted_keys = generate_encrypted_keys(proposal.end_time).await?;

    // this converts the public key to a big int
    let public_key = convert_to_public_key_big_int(&encrypted_keys.pub_key)?;

    //get random id in u16
    let proposal_id = generate_unique_random_id(db.clone()).await?;

    // this creates the base proof dto
    let aggregator_request_dto = AggregatorBaseDto {
        pk_enc: public_key,
        membership_root: parse_big_uint(&proposal.membership_root),
        proposal_id,
        init_nullifier_root: parse_big_uint(&proposal.nullifier),
    };

    log::info!("base proof dto {:?}", aggregator_request_dto);
    // this creates the base proof
    match create_base_proof(aggregator_request_dto).await {
        Ok(_) => (),
        Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
    };

    let start_time = proposal.start_time;
    let end_time = proposal.end_time;
    let id = ObjectId::new();
    let db_clone = db.clone();
    spawn(async move {
       schedule_event(&id.to_string(), db_clone, start_time, end_time).await;
    });
    
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
        status: ProposalStatus::Inactive,
        result: vec![],
        curr_agg_proof: None,
        is_aggregator_available: false,
        user_proof_array: vec![],
        user_proof_queue: vec![],
        id: Some(id),
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
    res: ProofFromAggregator,
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

    let snark = res.proof;
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
    log::debug!("num_round: {:?}", num_round);

    proposal.curr_agg_proof = Some(snark);

    let user_proof_queue = proposal.user_proof_queue.clone();
    if user_proof_queue.is_empty() {
        proposal.is_aggregator_available = true;
        let id = proposal.id.unwrap().to_string();
        match db.update(&id, proposal).await {
            Ok(_) => {
                log::info!("Proof submitted to proposal");
                Ok(())
            }
            Err(e) => Err(Error::new(ErrorKind::Other, e.to_string())),
        }
    } else {
        match submit_to_aggregator_from_queue(proposal, db.clone()).await {
            Ok(_) => {
                log::info!("Proof submitted to proposal from queue");
                Ok(())
            }
            Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
        }
    }
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

pub async fn get_result_on_proposal(
    db: web::Data<Repository<Proposal>>,
    id: &str,
) -> Result<Vec<String>, Error> {
    let mut proposal = get_proposal_by_id(db.clone(), id).await?;

    match proposal.status {
        ProposalStatus::Inactive => {
            return Err(Error::new(ErrorKind::Other, "Proposal is not started yet"));
        }
        ProposalStatus::Completed => {
            return Ok(proposal.result);
        }

        ProposalStatus::Active => {
            if proposal.end_time > Utc::now() {
                return Err(Error::new(ErrorKind::Other, "wait till end time"));
            } else {
                return Err(Error::new(ErrorKind::Other, "Votes are being calculated"))
            }
        }
    }
}

async fn call_reveal_result(result_dto: VoteResultDto) -> Result<Vec<String>, Error> {
    let addr = std::env::var("REVEAL_RESULT").unwrap_or_else(|_| "http://localhost:8080".into());
    let client = reqwest::Client::new();
    let response = match client.post(&addr).json(&result_dto).send().await {
        Ok(response) => response,
        Err(e) => return Err(Error::new(std::io::ErrorKind::Other, e.to_string())),
    };

    if response.status().is_success() {
        let json: Vec<String> = match response.json().await {
            Ok(json) => json,
            Err(e) => return Err(Error::new(std::io::ErrorKind::Other, e.to_string())),
        };
        Ok(json)
    } else {
        Err(Error::new(
            std::io::ErrorKind::Other,
            "Failed to get result",
        ))
    }
}

pub async fn get_all_proposals(
    db: web::Data<Repository<Proposal>>,
) -> Result<Vec<Proposal>, Error> {
    match db.find_all().await {
        Ok(result) => Ok(result),
        Err(e) => Err(Error::new(ErrorKind::Other, e.to_string())),
    }
}

pub async fn get_proposal_by_dao_id(
    db: web::Data<Repository<Proposal>>,
    dao_id: &str,
) -> Result<Vec<Proposal>, Error> {
    let dao_id = bson::Bson::String(dao_id.to_string());
    match db.find_all_by_field("daoId", dao_id).await {
        Ok(result) => Ok(result),
        Err(e) => Err(Error::new(ErrorKind::Other, e.to_string())),
    }
}

fn limbs_to_biguint(x: Vec<Fr>) -> BigUint {
    x.iter()
        .enumerate()
        .map(|(i, limb)| fe_to_biguint(limb) * BigUint::from(2u64).pow(88 * (i as u32)))
        .sum()
}
// call this function when vote queue is not empty
async fn submit_to_aggregator_from_queue(
    proposal: Proposal,
    proposal_db: web::Data<Repository<Proposal>>,
) -> Result<(), Error> {
    let mut proposal = proposal;
    let mut user_proof_queue = proposal.user_proof_queue.clone();
    let voter_snark = user_proof_queue.remove(0);
    proposal.user_proof_queue = user_proof_queue;

    let mut num_round: u64 = 0;

    if proposal.curr_agg_proof.clone().unwrap().instances[0]
        .last()
        .is_none()
    {
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

    let recurr_dto = AggregatorRecursiveDto {
        num_round: num_round as u16,
        voter: voter_snark.clone(),
        previous: proposal.curr_agg_proof.unwrap(),
    };

    match call_submit_to_aggregator(recurr_dto).await {
        Ok(_) => log::info!("recursive proof submited to rabbitMQ to aggregator"),
        Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
    }

    proposal.curr_agg_proof = None;
    proposal.is_aggregator_available = false;
    let proposal_id = proposal.id.unwrap().to_string();
    match proposal_db.update(&proposal_id, proposal).await {
        Ok(_) => {
            log::info!("Vote submitted to aggregator from queue");
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
    // let nullifier_inputs =
    //     update_nullifier_tree(proposal.curr_nullifier_preimages, nullifier, num_round + 1);
    let recurr_dto = AggregatorRecursiveDto {
        num_round: num_round as u16,
        voter: voter_snark.clone(),
        previous: previous_snark,
        // nullifier_tree_input: nullifier_inputs.1,
    };

    match call_submit_to_aggregator(recurr_dto).await {
        Ok(_) => {
            log::info!("recursive proof submited to RabbitMQ from voter as aggregator is available")
        }
        Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
    }

    proposal.curr_agg_proof = None;
    // proposal.curr_nullifier_preimages = nullifier_inputs.0;
    // proposal.curr_nullifier_root = proof.instances[0][37];
    // proposal.curr_nullifier_root = None;
    proposal.is_aggregator_available = false;

    match proposal_db.update(proposal_id, proposal).await {
        Ok(_) => {
            log::info!("Vote submitted to aggregator, as queue is empty");
            Ok(())
        }
        Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
    }
}

async fn call_submit_to_aggregator(
    dto: AggregatorRecursiveDto,
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

    log::info!("recursive proof sent to {:?}", &queue_name);
    Ok(())
}

async fn schedule_event(
    proposal_id: &str,
    db: web::Data<Repository<Proposal>>,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
) {
    let now = Utc::now();
    if start_time > now {
        let wait_duration = (start_time - now).to_std().unwrap();
        let sleep_time = Instant::now() + wait_duration;
        sleep_until(sleep_time).await;
    }
    if let Err(e) = handle_event_start(&proposal_id, db.clone()).await {
        log::error!("Failed to mark proposal as active: {}", e);
        return;
    }
    let end_duration = (end_time - Utc::now()).to_std().unwrap_or_default();

    // Wait until end time
    let end_sleep_time = Instant::now() + end_duration;

    sleep_until(end_sleep_time).await;

    if let Err(e) = handle_event_end(&proposal_id, db).await {
        log::error!("Failed to mark proposal as inactive: {}", e);
    }
}

async fn handle_event_start(
    proposal_id: &str,
    db: web::Data<Repository<Proposal>>,
) -> Result<(), Error> {
    update_proposal_status(proposal_id, db, ProposalStatus::Active).await
}

async fn handle_event_end(
    proposal_id: &str,
    db: web::Data<Repository<Proposal>>,
) -> Result<(), Error> {

    let mut proposal = get_proposal_by_id(db.clone(), proposal_id).await?;
    
    proposal.encrypted_keys.pvt_key =
                    decrypt_keys(proposal.encrypted_keys.pvt_key.clone()).await?;

    let snark = proposal.user_proof_array.clone();
    
    let (n, _) = match parse_public_key(&proposal.encrypted_keys.pub_key) {
        Ok((n, g)) => (n, g),
        Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
    };

    let vote_enc = snark.iter().map(|user_proof| {
        let instance = user_proof.instances.clone();
        instance[4..16]
            .chunks(4)
            .map(|v| limbs_to_biguint(v.to_vec()))
            .collect::<Vec<BigUint>>()
    }).collect::<Vec<Vec<BigUint>>>();

    let election_result = vote_enc.iter()
        .skip(1)
        .fold(vote_enc[0].clone(), |acc, vote| {
            acc.iter()
                .zip(vote.iter())
                .map(|(a, b)| paillier_add_native(&n, &a, &b))
                .collect()
        });

    let final_vote_in_string = election_result.iter().map(|v|v.to_string()).collect::<Vec<String>>();

    let vote_dto = VoteResultDto {
        pvt: proposal.encrypted_keys.pvt_key.clone(),
        vote: final_vote_in_string
    };

    proposal.result = call_reveal_result(vote_dto).await?;

    db.update(proposal_id, proposal)
        .await
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    update_proposal_status(proposal_id, db, ProposalStatus::Completed).await
}

async fn update_proposal_status(
    proposal_id: &str,
    db: web::Data<Repository<Proposal>>,
    status: ProposalStatus,
) -> Result<(), Error> {
    let proposal = db.find_by_id(proposal_id).await.map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
    let mut proposal = proposal.ok_or_else(|| Error::new(ErrorKind::NotFound, "Proposal not found"))?;

    proposal.status = status;

    db.update(proposal_id, proposal).await
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    Ok(())
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

    log::info!("base proof sent to {:?}", &queue_name);
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
