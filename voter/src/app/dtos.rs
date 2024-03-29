use halo2_base::halo2_proofs::halo2curves::bn256::Fr;
use serde::{Deserialize, Serialize};
use voter::EncryptionPublicKey;

#[derive(Clone, Serialize, Deserialize)]
pub struct VoterDto {
    // Backend
    pub proposal_id: u16,
    pub pk_enc: EncryptionPublicKey,
    pub membership_root: Fr,
    pub membership_proof: Vec<Fr>,
    pub membership_proof_helper: Vec<Fr>,
}