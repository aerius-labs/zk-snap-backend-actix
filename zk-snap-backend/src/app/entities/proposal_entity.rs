use aggregator::wrapper::common::Snark;
use halo2_base::{halo2_proofs::halo2curves::bn256::Fr, utils::ScalarField};
use mongodb::bson::oid::ObjectId;
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

use crate::app::dtos::proposal_dto::UserProofDto;

#[derive(Serialize, Deserialize, Clone)]
pub struct UserProof {
    #[serde(rename = "instances")]
    pub instances: Vec<Fr>,
    #[serde(rename = "proof")]
    pub proof: Vec<u8>,
}

impl UserProof {
    pub fn from_dto(dto: UserProofDto) -> Self {
        UserProof {
            instances: dto.instances.iter().map(|hex_str| {
                let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
                let biguint = BigUint::parse_bytes(hex_str.as_bytes(), 16)
                    .unwrap_or_else(|| panic!("Failed to parse hex string: {}", hex_str));
                Fr::from_bytes_le(
                    &biguint.to_bytes_le()
                )
            }).collect(),
            proof: dto.proof.clone(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Proposal {
    #[serde(rename = "_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    #[serde(rename = "proposalId")]
    pub proposal_id: u16,

    #[serde(rename = "creator")]
    pub creator: String,

    #[serde(rename = "daoName")]
    pub dao_name: String,

    #[serde(rename = "daoLogo")]
    pub dao_logo: String,

    #[serde(rename = "title")]
    pub title: String,

    #[serde(rename = "description")]
    pub description: String,

    #[serde(rename = "daoId")]
    pub dao_id: String,

    #[serde(rename = "startTime")]
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub start_time: chrono::DateTime<chrono::Utc>,

    #[serde(rename = "endTime")]
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub end_time: chrono::DateTime<chrono::Utc>,

    #[serde(rename = "encryptedKeys")]
    pub encrypted_keys: EncryptedKeys,

    #[serde(rename = "votingOptions")]
    pub voting_options: Vec<String>,

    #[serde(rename = "status")]
    pub status: ProposalStatus, // Could be refined to an Enum if desired

    #[serde(rename = "result")]
    pub result: Vec<String>,

    #[serde(rename = "currentAggProof")]
    pub curr_agg_proof: Option<Snark>,
    
    #[serde(rename = "IsAggregatorAvailable")]
    pub is_aggregator_available: bool,

    #[serde(rename = "userProofArray")]
    pub user_proof_array: Vec<UserProof>,

    #[serde(rename = "userProofQueue")]
    pub user_proof_queue: Vec<Snark>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EncryptedKeys {
    pub pub_key: String,
    pub pvt_key: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Vote {
    #[serde(rename = "proposalId")]
    pub proposal_id: String,

    // Simplified representation, assuming ZkProof is a struct defined elsewhere
    #[serde(rename = "userProof")]
    pub user_proof: ZkProof,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ProposalStatus {
    Active,
    Inactive,
    Completed,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ZkProof {
    #[serde(rename = "publicInput")]
    pub public_input: Vec<String>,

    #[serde(rename = "publicOutput")]
    pub public_output: Vec<String>,

    #[serde(rename = "maxProofsVerified")]
    pub max_proofs_verified: i32,

    #[serde(rename = "proof")]
    pub proof: String,
}
