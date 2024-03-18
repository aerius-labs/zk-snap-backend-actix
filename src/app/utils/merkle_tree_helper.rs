use hex::decode;
use halo2_base::{halo2_proofs::halo2curves::bn256::Fr, utils::BigPrimeField};
use pse_poseidon::Poseidon;
use std::io::Error;

fn spec_bytes_to_f<F: BigPrimeField>(bytes: &[u8; 32]) -> Result<[F;3], Error> {
    if bytes.len() != 32 {
        return Err(Error::new(std::io::ErrorKind::InvalidInput, "Invalid bytes length"));
    }
    let ve: Vec<F> = bytes.to_vec().chunks(11).map(|chunk|F::from_bytes_le(chunk)).collect();
    Ok([ve[0], ve[1], ve[2]])
}

pub fn preimage_to_leaf<F: BigPrimeField>(point: ([F;3], [F;3])) -> F { 
    let mut hasher = Poseidon::<F, 3, 2>::new(8, 57);
    hasher.update(&point.0);
    hasher.update(&point.1);
    hasher.squeeze_and_reset()
}

pub fn public_key_to_coordinates<F: BigPrimeField>(public_key_str: &str) -> Result<([F;3], [F;3]), Error> {
    let decoded_public_key = decode(public_key_str).ok().and_then(|bytes| {
        if bytes.len() == 65 && bytes[0] == 0x04 {
            Some(bytes[1..].to_vec())
        } else {
            None
        }
    });

    let x = decoded_public_key.as_ref().map(|bytes| {
        let mut x = [0u8; 32];
        x.copy_from_slice(&bytes[..32]);
        x
    }).unwrap();

    let y = decoded_public_key.as_ref().map(|bytes| {
        let mut y = [0u8; 32];
        y.copy_from_slice(&bytes[32..]);
        y
    }).unwrap();

    Ok((spec_bytes_to_f(&x)?, spec_bytes_to_f(&y)?))
}

pub fn from_members_to_leaf<F: BigPrimeField>(public_key_str: &[String]) -> Result<Vec<F>, Error> {
    let mut leaves = Vec::new();
    for pk_str in public_key_str {
        let coordinates = public_key_to_coordinates(pk_str).unwrap();
        let leaf = preimage_to_leaf(coordinates);
        leaves.push(leaf);
    }
    Ok(leaves)
}

pub fn encode_tree(input: &Vec<Vec<Fr>>) -> Vec<Vec<String>> {
    input
        .iter()
        .map(|inner_vec| {
            inner_vec
                .iter()
                .map(|fr| hex::encode(fr.to_bytes()))
                .collect()
        })
        .collect()
}

pub fn decode_tree(input: &Vec<Vec<String>>) -> Vec<Vec<Fr>> {
    input
        .iter()
        .map(|inner_vec| {
            inner_vec
                .iter()
                .map(|hex_str| {
                    let bytes = hex::decode(hex_str).expect("Failed to decode hex string");
                    assert!(bytes.len() == 32, "Invalid bytes length");

                    let mut bytes_array: [u8; 32] = [0; 32];
                    // copy vector to array
                    bytes_array.copy_from_slice(&bytes);
                    Fr::from_bytes(&bytes_array).expect("Failed to create Fr from bytes")
                })
                .collect()
        })
        .collect()
}