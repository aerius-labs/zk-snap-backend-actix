use std::io::Error;

use halo2_base::halo2_proofs::halo2curves::bn256::Fr;
use halo2_base::utils::BigPrimeField;
use indexed_merkle_tree_halo2::utils::IndexedMerkleTreeLeaf;
use num_bigint::BigUint;
use pse_poseidon::Poseidon;
use voter::merkletree::native::MerkleTree;

pub fn generate_default_leafs<F: BigPrimeField>(num_leaf: usize) -> Vec<IndexedMerkleTreeLeaf<F>> {
    //num_leaf should be power of 2
    // assert!(num_leaf != 0 && (num_leaf & (num_leaf - 1)) == 0);
    (0..num_leaf)
        .map(|_| IndexedMerkleTreeLeaf::<F> {
            val: F::from(0u64),
            next_val: F::from(0u64),
            next_idx: F::from(0u64),
        })
        .collect::<Vec<_>>()
}
pub fn insert_new_leaf<F: BigPrimeField>(
    leaves: Vec<IndexedMerkleTreeLeaf<F>>,
    new_val: F,
    new_val_idx: u64,
) -> (Vec<IndexedMerkleTreeLeaf<F>>, usize) {
    let mut nullifier_tree_preimages = leaves.clone();
    let mut low_leaf_idx = 0;
    for (i, node) in leaves.iter().enumerate() {
        if node.next_val == F::ZERO && i == 0 {
            nullifier_tree_preimages[i + 1].val = new_val;
            nullifier_tree_preimages[i].next_val = new_val;
            nullifier_tree_preimages[i].next_idx = F::from((i as u64) + 1);
            low_leaf_idx = i;
            break;
        }
        if node.val < new_val && (node.next_val > new_val || node.next_val == F::ZERO) {
            nullifier_tree_preimages[new_val_idx as usize].val = new_val;
            nullifier_tree_preimages[new_val_idx as usize].next_val =
                nullifier_tree_preimages[i].next_val;
            nullifier_tree_preimages[new_val_idx as usize].next_idx =
                nullifier_tree_preimages[i].next_idx;
            nullifier_tree_preimages[i].next_val = new_val;
            nullifier_tree_preimages[i].next_idx = F::from(new_val_idx);
            low_leaf_idx = i;
            break;
        }
    }
    (nullifier_tree_preimages, low_leaf_idx)
}
fn hash_nullifier_pre_images<F: BigPrimeField>(
    nullifier_tree_preimages: Vec<IndexedMerkleTreeLeaf<F>>,
) -> Vec<F> {
    let mut native_hasher = Poseidon::<F, 3, 2>::new(8, 57);
    nullifier_tree_preimages
        .iter()
        .map(|leaf| {
            native_hasher.update(&[leaf.val, leaf.next_val, leaf.next_idx]);
            native_hasher.squeeze_and_reset()
        })
        .collect::<Vec<_>>()
}

pub fn generate_merkle_root_of_nullifier<F: BigPrimeField>(
    nullifier_tree_preimages: Vec<IndexedMerkleTreeLeaf<F>>,
) -> Result<F, Error> {
    let mut hash = Poseidon::<F, 3, 2>::new(8, 57);
    let leaves = hash_nullifier_pre_images(nullifier_tree_preimages);
    let tree = match MerkleTree::new(&mut hash, leaves) {
        Ok(tree) => tree,
        Err(e) => return Err(Error::new(std::io::ErrorKind::InvalidInput, e.to_string())),
    };
    Ok(tree.get_root())
}
pub fn generate_nullifier_leaf_proof<F: BigPrimeField>(
    nullifier_tree_preimages: Vec<IndexedMerkleTreeLeaf<F>>,
    leaf_idx: usize,
) -> (Vec<F>, Vec<F>) {
    let mut hash = Poseidon::<F, 3, 2>::new(8, 57);
    let leaves = hash_nullifier_pre_images(nullifier_tree_preimages);
    let tree = MerkleTree::new(&mut hash, leaves).unwrap();
    tree.get_proof(leaf_idx)
}

// Functions to handle nullifier

pub fn nearest_power_of_two(num: u64) -> u64 {
    let mut power = 1;
    while power < num {
        power <<= 1;
    }
    power
}

pub fn generate_nullifier_root(
    size: u64,
) -> Result<(BigUint, Vec<IndexedMerkleTreeLeaf<Fr>>), Error> {
    let power_of_two = nearest_power_of_two(size);
    let leaves = generate_default_leafs::<Fr>(power_of_two as usize);
    let nullifier_tree_preimages = leaves.clone();
    let nullifier_root = generate_merkle_root_of_nullifier(nullifier_tree_preimages)?;
    Ok((
        BigUint::from_bytes_le(nullifier_root.to_bytes().as_slice()),
        leaves,
    ))
}
