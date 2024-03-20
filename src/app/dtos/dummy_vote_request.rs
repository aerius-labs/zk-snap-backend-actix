use serde::{Deserialize, Serialize};
use halo2_base::halo2_proofs::halo2curves::bn256::Fr;
use crate::app::utils::parse_string_pub_key::EncryptionPublicKey;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct VoterDto {
    // Backend
    pub proposal_id: u16,
    pub pk_enc: EncryptionPublicKey,
    pub membership_root: Fr,
    pub membership_proof: Vec<Fr>,
    pub membership_proof_helper: Vec<Fr>,
}