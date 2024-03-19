use halo2_base::{
    halo2_proofs::halo2curves::{bn256::Fr, secp256k1::Secp256k1Affine, serde::SerdeObject},
    utils::{fe_to_biguint, BigPrimeField, ScalarField},
};
use num_bigint::BigUint;
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

pub(crate) fn paillier_enc(pk_enc: EncryptionPublicKey, m: &BigUint) -> BigUint {
    let r = BigUint::from(0u64);
    let n = pk_enc.n.clone();
    let g = &pk_enc.g;
    let c = (g.modpow(m, &(n.clone() * &n.clone()))
        * r.modpow(&n.clone(), &(n.clone() * &n.clone())))
        % (n.clone() * n.clone());
    c
}

pub(crate) fn get_init_vote(pk_enc: EncryptionPublicKey) -> Vec<Fr> {
    let init_vote = (0..5)
        .map(|_| paillier_enc(pk_enc.clone(), &BigUint::from(0u64)))
        .collect::<Vec<BigUint>>();
    let init_vote = init_vote
        .iter()
        .flat_map(|x| biguint_to_88_bit_limbs(x.clone()))
        .collect::<Vec<Fr>>();
    init_vote
}

pub(crate) fn compressed_to_affine<F: BigPrimeField>(
    compressed: [F; 4],
) -> Option<Secp256k1Affine> {
    let mut bytes = Vec::with_capacity(33);

    let compressed_y_is_odd = compressed[0] != F::from(2u64);
    bytes.push(if compressed_y_is_odd { 0x03 } else { 0x02 });

    for (i, chunk) in compressed.iter().enumerate().skip(1) {
        let byte_chunk = chunk.to_bytes_le();
        let chunk_len = if i == 3 { 10 } else { 11 };
        bytes.extend_from_slice(&byte_chunk[..chunk_len]);
    }

    Secp256k1Affine::from_raw_bytes(&bytes)
}
