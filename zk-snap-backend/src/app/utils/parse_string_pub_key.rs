use std::io::Error;

use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

use hex::{decode, encode};
use sha3::{Digest, Keccak256};

#[derive(Deserialize, Debug)]
struct PublicKey {
    pub n: String,
    pub _n2: String,
    pub g: String,
}

pub fn parse_public_key(json_str: &str) -> Result<(BigUint, BigUint), Box<dyn std::error::Error>> {
    // Parse the JSON string into our struct
    let pubkey: PublicKey = serde_json::from_str(json_str)?;
    
    // Convert n and g from string to BigUint
    let n = BigUint::parse_bytes(pubkey.n.as_bytes(), 10)
        .ok_or("Failed to parse n as BigUint")?;
    let g = BigUint::parse_bytes(pubkey.g.as_bytes(), 10)
        .ok_or("Failed to parse g as BigUint")?;
    
    Ok((n, g))
}

#[derive(Serialize, Deserialize, Debug)]
struct PublicKeyContainer {
    pub_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EncryptionPublicKey {
    pub n: BigUint,
    pub g: BigUint,
}

impl EncryptionPublicKey {
    pub fn new(n: String, g: String) -> Result<Self, Error> {
        Ok(Self {
            n: match n.parse() {
                Ok(n) => n,
                Err(_) => {
                    return Err(Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Failed to parse n in public key",
                    ))
                }
            },
            g: match g.parse() {
                Ok(g) => g,
                Err(_) => {
                    return Err(Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Failed to parse g in public key",
                    ))
                }
            },
        })
    }
}

// Function to convert pub_key string to PublicKeyBigInt
pub fn convert_to_public_key_big_int(pub_key_json: &str) -> Result<EncryptionPublicKey, Error> {
    // Deserialize the `pub_key` JSON string into the PublicKey struct
    let pub_key: PublicKey = serde_json::from_str(pub_key_json)?;

    // Use the `n` and `g` to create PublicKeyBigInt
    Ok(EncryptionPublicKey::new(pub_key.n, pub_key.g)?)
}

pub fn public_key_to_eth_address(public_key_hex: &str) -> Result<String, hex::FromHexError> {
    // Decode the hex string to bytes, skipping the first 2 characters to remove the "0x" prefix
    let public_key = decode(&public_key_hex[2..])?;

    // Ensure the public key is in the correct format (uncompressed, without the 0x04 prefix if present)
    let public_key = if public_key.starts_with(&[0x04]) {
        &public_key[1..]
    } else {
        &public_key
    };

    // Hash the public key using Keccak-256
    let mut hasher = Keccak256::new();
    hasher.update(public_key);
    let result = hasher.finalize();

    // Take the last 20 bytes and convert them to a hex string with '0x' prefix
    let address = &result[12..];
    Ok(format!("0x{}", encode(address)))
}
