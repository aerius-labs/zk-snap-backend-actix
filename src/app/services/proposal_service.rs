use chrono::{DateTime, Utc};
use dotenv::dotenv;
use mongodb::bson::oid::ObjectId;
use reqwest::Client;
use std::env;
use std::io::{Error, ErrorKind};
use tokio::time::{sleep_until, Instant};

use super::dao_service;
use crate::app::entities::proposal_entity::EncryptedKeys;
use crate::app::{
    dtos::proposal_dto::{CreateProposalDto, DecryptRequest, DecryptResponse},
    entities::{dao_entity::Dao, proposal_entity::Proposal},
    repository::generic_repository::Repository,
};
use actix_web::web;

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

    let proposal = Proposal {
        creator: proposal.creator,
        title: proposal.title,
        description: proposal.description,
        dao_id: proposal.dao_id,
        start_time: proposal.start_time,
        end_time: proposal.end_time,
        encrypted_keys,
        voting_options: proposal.voting_options,
        status: "Pending".to_string(),
        result: vec![],
        id: Some(ObjectId::new()),
    };

    // this schedules the event to handle the end of the proposal
    let proposal_id = proposal.id.unwrap().to_string();
    schedule_event(&proposal_id, db.clone(), proposal.end_time).await;

    match db.create(proposal).await {
        Ok(result) => Ok(result),
        Err(e) => Err(Error::new(ErrorKind::Other, e.to_string())),
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
                let json: DecryptResponse = resp.json().await.unwrap();
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

async fn generate_encrypted_keys(end_time: DateTime<Utc>) -> Result<EncryptedKeys, Error> {
    dotenv().ok();
    let url = env::var("ENCRYPTION_URL").expect("URL is not set");
    let formatted_date_time = end_time.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

    let url = url.to_owned() + &formatted_date_time;

    let response = reqwest::Client::new().get(&url).send().await.unwrap();

    if response.status().is_success() {
        let json: EncryptedKeys = response.json().await.unwrap();
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
