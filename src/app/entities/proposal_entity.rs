use aggregator::wrapper::common::Snark;
use halo2_base::halo2_proofs::halo2curves::bn256::Fr;
use indexed_merkle_tree_halo2::utils::IndexedMerkleTreeLeaf;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
pub struct Proposal {
    #[serde(rename = "_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    #[serde(rename = "proposalId")]
    pub proposal_id: u16,

    #[serde(rename = "creator")]
    pub creator: String,

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
    pub status: String, // Could be refined to an Enum if desired

    #[serde(rename = "result")]
    pub result: Vec<String>,

    #[serde(rename = "currentAggProof")]
    pub curr_agg_proof: Snark,

    #[serde(rename = "PreviousNullifierRoot")]
    pub curr_nullifier_root: Fr,

    #[serde(rename = "PreviousNullifierLeaves")]
    pub curr_nullifier_preimages: Vec<IndexedMerkleTreeLeaf<Fr>>,
    // #[serde(rename = "userProofQueue")]
    // pub user_proof_queue: Vec<Vote>,
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
