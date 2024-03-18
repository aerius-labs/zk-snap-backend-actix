use num_bigint::BigUint;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AggregatorBaseDto {
    pub pk_enc: [BigUint; 2],
    pub membership_root: BigUint,
    pub proposal_id: u16,
    pub init_nullifier_root: BigUint,
}

#[derive(Debug, Deserialize)]
pub struct AggregatorRecursiveDto;
