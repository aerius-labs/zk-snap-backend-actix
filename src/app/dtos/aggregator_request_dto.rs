use crate::app::utils::parse_string_pub_key::EncryptionPublicKey;
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AggregatorBaseDto {
    pub pk_enc: EncryptionPublicKey,
    pub membership_root: BigUint,
    pub proposal_id: u16,
    pub init_nullifier_root: BigUint,
}
