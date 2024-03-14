use chrono::{DateTime, Utc};
use dotenv::dotenv;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::env;
use std::io::{Error, ErrorKind};

use super::dao_service;
use crate::app::entities::proposal_entity::EncryptedKeys;
use crate::app::{
    dtos::proposal_dto::CreateProposalDto,
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

    match db.create(proposal).await {
        Ok(result) => Ok(result),
        Err(e) => Err(Error::new(ErrorKind::Other, e.to_string())),
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
