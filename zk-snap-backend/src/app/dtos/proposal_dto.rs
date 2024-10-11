use chrono::Utc;
use halo2_base::halo2_proofs::halo2curves::bn256::Fr;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use crate::app::entities::proposal_entity::{Proposal, ProposalStatus};
//TODO FIX Complete file
fn validate_title_length(value: &str) -> Result<(), ValidationError> {
    if value.len() > 100 {
        Err(ValidationError::new(
            "Title should not be empty and not greater than 100 characters",
        ))
    } else {
        Ok(())
    }
}

fn validate_description_length(value: &str) -> Result<(), ValidationError> {
    if value.len() > 500 {
        Err(ValidationError::new(
            "Description should not be empty and not greater than 500 characters",
        ))
    } else {
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Validate, Debug)]
pub struct CreateProposalDto {
    #[validate(length(min = 1))]
    pub creator: String,

    #[validate(custom = "validate_title_length")]
    pub title: String,

    #[validate(custom = "validate_description_length")]
    pub description: String,

    #[validate(length(min = 1))]
    pub dao_id: String,

    pub start_time: chrono::DateTime<Utc>,

    pub end_time: chrono::DateTime<Utc>,

    // Assume default values are provided if empty
    #[serde(default = "default_voting_options")]
    pub voting_options: Vec<String>,

    pub membership_root: String,
    pub membership_proof: String,
    pub nullifier: String,
    // pub membership_proof_helper: Vec<Fr>,
}

#[derive(Serialize, Deserialize)]
pub struct ProposalResponseDto {
    pub proposal_id: String,
    pub dao_name: String,
    pub creator: String,
    pub dao_logo: String,
    pub title: String,
    pub status: ProposalStatus,
    pub start_time: chrono::DateTime<Utc>,
    pub end_time: chrono::DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct ProposalByIdResponseDto {
    pub dao_name: String, 
    pub creator_address: String,
    pub proposal_id: String,
    pub proposal_name: String,
    pub proposal_description: String,
    pub proposal_tile: String,
    pub start_time: chrono::DateTime<Utc>,
    pub end_time: chrono::DateTime<Utc>,
    
}

#[derive(Serialize, Deserialize)]
pub struct MerkleProofVoter {
    pub proof: Vec<Fr>,
    pub helper: Vec<Fr>,
}

impl MerkleProofVoter {
    pub fn new(proof: Vec<Fr>, helper: Vec<Fr>) -> Self {
        MerkleProofVoter {
            proof: proof,
            helper: helper,
        }
    }
}

// Assuming that your encryption service expects a JSON with "pvt" field
#[derive(Serialize)]
pub struct DecryptRequest {
    pub pvt: String,
}

#[derive(Deserialize)]
pub struct DecryptResponse {
    // Adjust according to the actual response structure
    pub value: String,
}

#[derive(Serialize, Deserialize)]
pub struct VoteResultDto {
    pub pvt: String,
    pub vote: Vec<String>,
}

#[derive(Serialize, Deserialize, Validate, Debug)]
pub struct UpdateProposalDto {
    #[validate(custom = "validate_title_length")]
    pub title: String,

    #[validate(custom = "validate_description_length")]
    pub description: String,

    pub start_time: chrono::DateTime<Utc>,

    pub end_time: chrono::DateTime<Utc>,

    // Validation for uniqueness would need to be done manually or via a custom validator
    pub voting_options: Vec<String>,
}

fn default_voting_options() -> Vec<String> {
    vec!["Yes".to_string(), "No".to_string()]
}

// You will need to add the `validator` and `serde_with` crates to your `Cargo.toml`
