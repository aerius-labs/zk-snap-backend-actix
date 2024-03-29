use num_bigint::BigUint;
use voter::EncryptionPublicKey;


pub(crate) fn paillier_enc(pk_enc: EncryptionPublicKey, m: &BigUint, r: &BigUint) -> BigUint {
    let n2 = pk_enc.n.clone() * pk_enc.n.clone();
    let gm = pk_enc.g.modpow(m, &n2);
    let rn = r.modpow(&pk_enc.n, &n2);
    (gm * rn) % n2
}
