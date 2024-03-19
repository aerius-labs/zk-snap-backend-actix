use std::io::Error;

use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
struct PublicKey {
    pub n: String,
    pub _n2: String,
    pub g: String,
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
