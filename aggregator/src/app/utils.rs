use std::io::{Error, ErrorKind};

use halo2_base::{
    halo2_proofs::{
        arithmetic::CurveAffine,
        halo2curves::{
            bn256::Fr,
            ff::PrimeField,
            secp256k1::{Fp, Secp256k1Affine},
        },
    },
    utils::{biguint_to_fe, fe_to_biguint, BigPrimeField, ScalarField},
};
use num_bigint::{BigUint, RandBigInt};
use num_traits::Num;
use rand::thread_rng;
use voter::EncryptionPublicKey;

pub(crate) fn biguint_to_88_bit_limbs(x: BigUint) -> Vec<Fr> {
    let mut output = Vec::<Fr>::new();
    output.extend(x.to_bytes_le().chunks(11).map(Fr::from_bytes_le));
    output
}

pub(crate) fn limbs_to_biguint(x: Vec<Fr>) -> BigUint {
    x.iter()
        .enumerate()
        .map(|(i, limb)| fe_to_biguint(limb) * BigUint::from(2u64).pow(88 * (i as u32)))
        .sum()
}

pub(crate) fn paillier_enc(pk_enc: EncryptionPublicKey, m: &BigUint, r: &BigUint) -> BigUint {
    let n = pk_enc.n.clone();
    let g = &pk_enc.g;
    let c = (g.modpow(m, &(n.clone() * &n.clone()))
        * r.modpow(&n.clone(), &(n.clone() * &n.clone())))
        % (n.clone() * n.clone());
    c
}

pub(crate) fn get_init_vote(pk_enc: EncryptionPublicKey) -> Vec<Fr> {
    let init_vote = (0..5)
        .map(|_| {
            paillier_enc(
                pk_enc.clone(),
                &BigUint::from(0u64),
                &thread_rng().gen_biguint(176),
            )
        })
        .collect::<Vec<BigUint>>();
    let init_vote = init_vote
        .iter()
        .flat_map(|x| biguint_to_88_bit_limbs(x.clone()))
        .collect::<Vec<Fr>>();
    init_vote
}

pub(crate) fn compressed_to_affine<F: BigPrimeField>(
    compressed: [F; 4],
) -> Result<Secp256k1Affine, Error> {
    let compressed_y_is_odd = compressed[0] != F::from(2u64);

    let mut x_bytes = Vec::with_capacity(32);
    for (i, chunk) in compressed.iter().enumerate().skip(1) {
        let byte_chunk = chunk.to_bytes_le();
        let chunk_len = if i == 3 { 10 } else { 11 };
        x_bytes.extend_from_slice(&byte_chunk[..chunk_len]);
    }
    let x_bytes: [u8; 32] = x_bytes[..32].try_into().expect("Slice length must be 32");
    let x = fe_to_biguint::<Fp>(&Fp::from_bytes(&x_bytes).unwrap());

    let modulus = BigUint::from_str_radix(&Fp::MODULUS.to_string()[2..], 16).unwrap();
    let y2 = (x.modpow(&BigUint::from(3u64), &modulus) + BigUint::from(7u64)) % &modulus;
    let mut y = y2.modpow(
        &((modulus.clone() + BigUint::from(1u64)) / BigUint::from(4u64)),
        &modulus,
    );
    if y.bit(0) != compressed_y_is_odd {
        y = modulus - y;
    }

    let pt = Secp256k1Affine::from_xy(biguint_to_fe::<Fp>(&x), biguint_to_fe::<Fp>(&y)).unwrap();

    match format!("{:?}", pt.is_on_curve()).as_str() {
        "Choice(1)" => Ok(pt),
        _ => Err(Error::new(ErrorKind::InvalidData, "Invalid point")),
    }
}
