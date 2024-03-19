use aggregator::{state_transition::IndexedMerkleTreeInput, wrapper::common::Snark};
use halo2_base::halo2_proofs::halo2curves::bn256::Fr;
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use voter::EncryptionPublicKey;

#[derive(Debug, Serialize, Deserialize)]
pub struct AggregatorBaseDto {
    pub pk_enc: EncryptionPublicKey,
    pub membership_root: BigUint,
    pub proposal_id: u16,
    pub init_nullifier_root: BigUint,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AggregatorRecursiveDto {
    pub num_round: u16,
    pub voter: Snark,
    pub previous: Snark,
    pub nullifier_tree_input: IndexedMerkleTreeInput<Fr>,
}