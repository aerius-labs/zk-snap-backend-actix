use std::io::{Error, ErrorKind};

use aggregator::wrapper::{common::Snark, recursion::RecursionCircuit};
use halo2_base::{
    gates::circuit::BaseCircuitParams,
    halo2_proofs::{
        halo2curves::{
            bn256::{Bn256, Fr},
            ff::PrimeField,
        },
        poly::kzg::commitment::ParamsKZG,
    },
    utils::{biguint_to_fe, fs::gen_srs, ScalarField},
};
use num_bigint::BigUint;
use num_traits::Num;

use super::dtos::AggregatorBaseDto;

fn biguint_to_88_bit_limbs(x: BigUint) -> Vec<Fr> {
    let mut output = Vec::<Fr>::new();
    output.extend(x.to_bytes_le().chunks(11).map(Fr::from_bytes_le));
    output
}

fn paillier_enc(pk_enc: [BigUint; 2], m: &BigUint) -> BigUint {
    let r = BigUint::from(0u64);
    let n = pk_enc[0].clone();
    let g = &pk_enc[1];
    let c = (g.modpow(m, &(n.clone() * &n.clone()))
        * r.modpow(&n.clone(), &(n.clone() * &n.clone())))
        % (n.clone() * n.clone());
    c
}

fn get_init_vote(pk_enc: [BigUint; 2]) -> Vec<Fr> {
    let init_vote = (0..5)
        .map(|_| paillier_enc(pk_enc.clone(), &BigUint::from(0u64)))
        .collect::<Vec<BigUint>>();
    let init_vote = init_vote
        .iter()
        .flat_map(|x| biguint_to_88_bit_limbs(x.clone()))
        .collect::<Vec<Fr>>();
    init_vote
}

fn generate_base_witness(
    input: AggregatorBaseDto,
) -> Result<(ParamsKZG<Bn256>, BaseCircuitParams, Vec<Fr>), Error> {
    let k: usize = 22;
    let params = gen_srs(k as u32);
    let config = BaseCircuitParams {
        k,
        num_advice_per_phase: vec![4],
        num_lookup_advice_per_phase: vec![1, 0, 0],
        num_fixed: 1,
        lookup_bits: Some(k - 1),
        num_instance_columns: 1,
    };

    if input.pk_enc[0] <= BigUint::from(0u64) || input.pk_enc[1] <= BigUint::from(0u64) {
        return Err(Error::new(ErrorKind::InvalidData, "Invalid Encryption Key"));
    }
    let pk_enc = input.pk_enc;

    let modulus = BigUint::from_str_radix(&Fr::MODULUS.to_string()[2..], 16).unwrap();

    if input.membership_root >= modulus {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Membership root is greater than fr modulus",
        ));
    }
    let membership_root = biguint_to_fe::<Fr>(&input.membership_root);

    if (input.proposal_id as u32) >= (2u32).pow(16) {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Proposal id is greater than 2^16",
        ));
    }
    let proposal_id = Fr::from_u128(input.proposal_id as u128);

    if input.init_nullifier_root >= modulus {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Init nullifier root is greater than fr modulus",
        ));
    }
    let init_nullifier_root = biguint_to_fe::<Fr>(&input.init_nullifier_root);

    let pk_enc_n = biguint_to_88_bit_limbs(pk_enc[0].clone());
    let pk_enc_g = biguint_to_88_bit_limbs(pk_enc[1].clone());

    let init_vote = get_init_vote(pk_enc.clone());

    let mut base_instances = vec![Fr::zero()]; // preprocessed_digest
    base_instances.extend(pk_enc_n);
    base_instances.extend(pk_enc_g);
    base_instances.extend(init_vote);
    base_instances.extend([
        init_nullifier_root,
        init_nullifier_root,
        membership_root,
        proposal_id,
        Fr::from(0),
    ]);

    Ok((params, config, base_instances))
}

pub async fn generate_base_proof(input: AggregatorBaseDto) -> Result<Snark, Error> {
    let (params, config, base_instances) = generate_base_witness(input)?;
    let base_snark = RecursionCircuit::initial_snark(&params, None, config, base_instances);

    Ok(base_snark)
}

pub async fn generate_recursive_proof() -> Result<(), Error> {
    unimplemented!()
}
