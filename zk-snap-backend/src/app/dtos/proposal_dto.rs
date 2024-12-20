use bson::{doc, oid::ObjectId, Document};
use chrono::Utc;
use halo2_base::halo2_proofs::halo2curves::bn256::Fr;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use crate::app::{entities::proposal_entity::{EncryptedKeys, ProposalStatus}, repository::traits::{Projectable, ProjectableByField}};

// Custom validation function
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

/// Data transfer object for creating a proposal
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

    #[serde(default = "default_voting_options")]
    pub voting_options: Vec<String>,

    pub membership_root: String,
    pub membership_proof: String,
    pub nullifier: String,
}

/// Data transfer object for updating a proposal
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UserProofDto {
    pub instances: Vec<String>,
    pub proof: Vec<u8>,
}

/// Data transfer object for getting all proposal
#[derive(Serialize, Deserialize)]
pub struct ProposalResponseDto {
    pub proposal_id: String,  // Changed to String to match output format
    pub dao_name: String,
    pub creator: String,
    pub dao_logo: String,
    pub title: String,
    pub status: ProposalStatus,
    pub start_time: chrono::DateTime<Utc>,
    pub end_time: chrono::DateTime<Utc>,
}

/// Implement the Projectable trait for ProposalResponseDto
impl Projectable for ProposalResponseDto {
    fn get_projection_pipeline(_: Option<ObjectId>) -> Vec<Document> {
        vec![
            doc! {
                "$project": {
                    "_id": 1,
                    "daoName": 1,
                    "creator": 1,
                    "daoLogo": 1,
                    "title": 1,
                    "status": 1,
                    "startTime": 1,
                    "endTime": 1,
                }
            },
            doc! {
                "$addFields": {
                    "proposal_id": { "$toString": "$_id" },
                    "dao_name": "$daoName",
                    "dao_logo": "$daoLogo",
                    "start_time": {
                        "$dateToString": {
                            "format": "%Y-%m-%dT%H:%M:%S.%LZ",
                            "date": "$startTime"
                        }
                    },
                    "end_time": {
                        "$dateToString": {
                            "format": "%Y-%m-%dT%H:%M:%S.%LZ",
                            "date": "$endTime"
                        }
                    },
                }
            },
            doc! {
                "$project": {
                    "_id": 0,
                    "daoName": 0,
                    "daoLogo": 0,
                    "startTime": 0,
                    "endTime": 0,
                }
            }
        ]
    }

}

/// Implement the ProjectableByField trait for ProposalResponseDto
impl ProjectableByField for ProposalResponseDto {
    fn get_projection_pipeline_by_field(field: &str) -> Vec<Document> {
        vec![
            doc! {
                "$match": {
                    "daoId": field
                }
            },
            doc! {
                "$project": {
                    "_id": 1,
                    "daoName": 1,
                    "creator": 1,
                    "daoLogo": 1,
                    "title": 1,
                    "status": 1,
                    "startTime": 1,
                    "endTime": 1,
                }
            },
            doc! {
                "$addFields": {
                    "proposal_id": { "$toString": "$_id" },
                    "dao_name": "$daoName",
                    "dao_logo": "$daoLogo",
                    "start_time": {
                        "$dateToString": {
                            "format": "%Y-%m-%dT%H:%M:%S.%LZ",
                            "date": "$startTime"
                        }
                    },
                    "end_time": {
                        "$dateToString": {
                            "format": "%Y-%m-%dT%H:%M:%S.%LZ",
                            "date": "$endTime"
                        }
                    },
                }
            },
            doc! {
                "$project": {
                    "_id": 0,
                    "daoName": 0,
                    "daoLogo": 0,
                    "startTime": 0,
                    "endTime": 0,
                }
            }
        ]
    }
}

/// Data transfer object for getting a proposal by ID
#[derive(Serialize, Deserialize)]
pub struct ProposalByIdResponseDto {
    pub dao_name: String,
    pub dao_logo: String,
    pub dao_id: String,
    pub creator_address: String,
    pub proposal_id: String,
    pub proposal_name: String,
    pub proposal_status: ProposalStatus,
    pub proposal_description: String,
    pub start_time: chrono::DateTime<Utc>,
    pub end_time: chrono::DateTime<Utc>,
    pub encrypted_keys: EncryptedKeys
}

/// Data transfer object from the database
#[derive(Serialize, Deserialize, Debug)]
pub struct ProposalProjectedFields {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    #[serde(rename = "daoLogo")]
    pub dao_logo: String,
    #[serde(rename = "daoName")]
    pub dao_name: String,
    #[serde(rename = "daoId")]
    pub dao_id: String,
    pub creator: String,
    pub status: ProposalStatus,
    pub description: String,
    pub title: String,
    #[serde(rename = "startTime", with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub start_time: chrono::DateTime<Utc>,
    #[serde(rename = "endTime", with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub end_time: chrono::DateTime<Utc>,
    #[serde(rename = "encryptedKeys")]
    pub encrypted_keys: EncryptedKeys
}

/// Implement the Projectable trait for ProposalProjectedFields
impl Projectable for ProposalProjectedFields {
    fn get_projection_pipeline(obj_id: Option<ObjectId>) -> Vec<Document> {
        let obj_id = match obj_id {
            Some(id) => id,
            None => ObjectId::new(),
        };
        vec![
            doc! {
                "$match": {
                    "_id": obj_id
                }
            },
            doc! {
                "$project": {
                    "_id": 1,
                    "daoName": 1,
                    "daoLogo": 1,
                    "daoId": 1,
                    "creator": 1,
                    "status": 1,
                    "description": 1,
                    "title": 1,
                    "startTime": 1,  // MongoDB will handle DateTime conversion
                    "endTime": 1,
                    "encryptedKeys": 1
                }
            }
        ]
    }
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
