use std::{
    env, fs,
    io::{BufReader, Error, ErrorKind},
};

use aggregator::{
    state_transition::{StateTransitionCircuit, StateTransitionInput},
    wrapper::{
        common::{gen_snark, Snark},
        recursion::RecursionCircuit,
    },
};
use halo2_base::{
    gates::circuit::{builder::BaseCircuitBuilder, BaseCircuitParams},
    halo2_proofs::{
        dev::MockProver, halo2curves::{
            bn256::{Bn256, Fr, G1Affine},
            ff::PrimeField,
        }, plonk::ProvingKey, poly::kzg::commitment::ParamsKZG
    },
    utils::{biguint_to_fe, fs::gen_srs},
};
use num_bigint::BigUint;
use num_traits::Num;
use voter::{EncryptionPublicKey, CircuitExt};

use crate::app::utils::{compressed_to_affine, limbs_to_biguint};

use super::{
    dtos::{AggregatorBaseDto, AggregatorRecursiveDto},
    utils::{biguint_to_88_bit_limbs, get_init_vote},
};

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

    if input.pk_enc.n <= BigUint::from(0u64) || input.pk_enc.g <= BigUint::from(0u64) {
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

    let pk_enc_n = biguint_to_88_bit_limbs(pk_enc.n.clone());
    let pk_enc_g = biguint_to_88_bit_limbs(pk_enc.g.clone());

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

fn generate_state_transition_proof(input: AggregatorRecursiveDto) -> Result<Snark, Error> {
    let voter = input.voter;
    let previous = input.previous;

    // pk_enc
    let pk_enc = EncryptionPublicKey {
        n: limbs_to_biguint(voter.instances[0][0..2].to_vec()),
        g: limbs_to_biguint(voter.instances[0][2..4].to_vec()),
    };

    let incoming_vote: Vec<BigUint> = (0..5)
        .map(|i| {
            let start = (i + 1) * 4;
            let end = start + 4;
            log::info!("start={}, end={}", start, end);
            limbs_to_biguint(voter.instances[0][start..end].to_vec())
        })
        .collect();
    log::info!("voter instances: {:?}", voter.instances[0]);
    log::info!("Incoming vote: {:?}", incoming_vote);

    let prev_vote: Vec<BigUint> = (0..5)
        .map(|i| {
            let start = 17 + 4*i;
            let end = start + 4;
            log::info!("start={}, end={}", start, end);
            limbs_to_biguint(previous.instances[0][start..end].to_vec())
        })
        .collect();
    log::info!("previous instances: {:?}", previous.instances[0]);
    log::info!("Prev vote: {:?}", prev_vote);
    let nullifier = compressed_to_affine::<Fr>([
        voter.instances[0][24],
        voter.instances[0][25],
        voter.instances[0][26],
        voter.instances[0][27],
    ])
    .map_err(|error| Error::new(ErrorKind::Other, error.to_string()))?;

    let state_transition_input = StateTransitionInput {
        pk_enc,
        incoming_vote,
        prev_vote,
        nullifier_tree: input.nullifier_tree_input,
        nullifier,
    };

    const K: usize = 15;
    let params = gen_srs(K as u32);
    let config = BaseCircuitParams {
        k: K,
        num_advice_per_phase: vec![3],
        num_lookup_advice_per_phase: vec![1, 0, 0],
        num_fixed: 1,
        lookup_bits: Some(K - 1),
        num_instance_columns: 1,
    };

    let circuit = StateTransitionCircuit::<Fr>::new(config.clone(), state_transition_input);

    let build_dir = env::current_dir()
        .map_err(|error| Error::new(ErrorKind::Other, error.to_string()))?
        .join("aggregator")
        .join("build");
    fs::create_dir_all(&build_dir).unwrap();

    let file = fs::read(build_dir.join("state_transition_pk.bin"))
        .map_err(|error| Error::new(ErrorKind::Other, error.to_string()))?;

    let pk_reader = &mut BufReader::new(file.as_slice());
    let pk = ProvingKey::<G1Affine>::read::<BufReader<&[u8]>, BaseCircuitBuilder<Fr>>(
        pk_reader,
        halo2_base::halo2_proofs::SerdeFormat::RawBytesUnchecked,
        config,
    )
    .map_err(|error| Error::new(ErrorKind::Other, error.to_string()))?;

    Ok(gen_snark(&params, &pk, circuit))
}

pub async fn generate_base_proof(input: AggregatorBaseDto) -> Result<Snark, Error> {
    let (params, config, base_instances) = generate_base_witness(input)?;
    let base_snark = RecursionCircuit::initial_snark(&params, None, config, base_instances);

    Ok(base_snark)
}

pub async fn generate_recursive_proof(input: AggregatorRecursiveDto) -> Result<Snark, Error> {
    let state_transition_snark = generate_state_transition_proof(input.clone())?;

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

    let build_dir = env::current_dir()
        .map_err(|error| Error::new(ErrorKind::Other, error.to_string()))?
        .join("aggregator")
        .join("build");
    fs::create_dir_all(&build_dir).unwrap();
    let file = fs::read(build_dir.join("recursion_pk.bin")).unwrap();
    let recursion_pk_reader = &mut BufReader::new(file.as_slice());
    let recursion_pk = ProvingKey::<G1Affine>::read::<BufReader<&[u8]>, BaseCircuitBuilder<Fr>>(
        recursion_pk_reader,
        halo2_base::halo2_proofs::SerdeFormat::RawBytesUnchecked,
        config.clone(),
    )
    .unwrap();

    let circuit = RecursionCircuit::new(
        halo2_base::gates::circuit::CircuitBuilderStage::Prover,
        &params,
        input.voter,
        state_transition_snark,
        input.previous,
        input.num_round as usize,
        config,
    );

    log::info!("Running mock prover");
    MockProver::run(22, &circuit, circuit.instances()).unwrap().verify().unwrap();
    log::info!("Mock prover finished");

    Ok(gen_snark(&params, &recursion_pk, circuit))
}