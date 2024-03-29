use std::{
    env, fs,
    io::{BufReader, Error, ErrorKind},
};

use aggregator::wrapper::common::{gen_snark, Snark};
   
use halo2_base::{
    gates::circuit::{builder::BaseCircuitBuilder, BaseCircuitParams},
    halo2_proofs::{
        arithmetic::Field, halo2curves::{
            bn256::{Fr, G1Affine},
            secp256k1::{Fq, Secp256k1Affine},
        }, plonk::ProvingKey,
    },
    utils::{fe_to_biguint, fs::gen_srs},
};
use num_bigint::{BigUint, RandBigInt};
use rand::{rngs::OsRng, thread_rng};
use voter::{ VoterCircuit, VoterCircuitInput};


use super::{
    dtos::VoterDto,
    utils::paillier_enc,
};


pub async fn generate_voter_proof(input: VoterDto) -> Result<Snark, Error> {
    let k: usize = 15;
    let config = BaseCircuitParams {
        k: 15,
        num_advice_per_phase: vec![1],
        num_lookup_advice_per_phase: vec![1, 0, 0],
        num_fixed: 1,
        lookup_bits: Some(14),
        num_instance_columns: 1,
    };
    //TODO: only read params
    let params = gen_srs(k as u32);

    let build_dir = env::current_dir()
        .map_err(|error| Error::new(ErrorKind::Other, error.to_string()))?
        .join("aggregator")
        .join("build");
    fs::create_dir_all(&build_dir).unwrap();
    // TODO: Remove unwrap
    let file = fs::read(build_dir.join("voter_pk.bin")).unwrap();
    let pk_reader = &mut BufReader::new(file.as_slice());
    let pk = ProvingKey::<G1Affine>::read::<BufReader<&[u8]>, BaseCircuitBuilder<Fr>>(
        pk_reader,
        halo2_base::halo2_proofs::SerdeFormat::RawBytesUnchecked,
        config.clone(),
    )
    .unwrap();

    let vote = [Fr::one(), Fr::zero(), Fr::zero(), Fr::zero(), Fr::zero()].to_vec();

    let mut rng = thread_rng();
    let r_enc = [
        rng.gen_biguint(176),
        rng.gen_biguint(176),
        rng.gen_biguint(176),
        rng.gen_biguint(176),
        rng.gen_biguint(176),
    ]
    .to_vec();

    let vote_enc = (0..5)
        .map(|i| paillier_enc(input.pk_enc.clone(), &fe_to_biguint(&vote[i]), &r_enc[i]))
        .collect::<Vec<BigUint>>();

    let pk_voter = Secp256k1Affine::random(OsRng);
    let nullifier = Secp256k1Affine::random(OsRng);
    let s_nullifier = Fq::random(OsRng);
    let c_nullifier = Fq::random(OsRng);

    let voter_input = VoterCircuitInput::<Fr> {
        membership_root: input.membership_root,
        pk_enc: input.pk_enc,
        nullifier,
        proposal_id: Fr::from(input.proposal_id as u64),
        vote_enc,
        s_nullifier,
        vote,
        r_enc,
        pk_voter,
        c_nullifier,
        membership_proof: input.membership_proof,
        membership_proof_helper: input.membership_proof_helper,
    };

    let circuit = VoterCircuit::new(config, voter_input);

    Ok(gen_snark(&params, &pk, circuit))
}
