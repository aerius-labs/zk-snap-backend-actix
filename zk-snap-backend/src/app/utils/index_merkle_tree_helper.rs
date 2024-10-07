use aggregator::state_transition::IndexedMerkleTreeInput;
use halo2_base::utils::BigPrimeField;
use indexed_merkle_tree_halo2::utils::IndexedMerkleTreeLeaf;
use pse_poseidon::Poseidon;
use voter::merkletree::native::MerkleTree;

pub fn generate_default_leafs<F: BigPrimeField>(num_leaf: usize) -> Vec<IndexedMerkleTreeLeaf<F>> {
    //num_leaf should be power of 2
    assert!(num_leaf != 0 && (num_leaf & (num_leaf - 1)) == 0);
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
) -> F {
    let mut hash = Poseidon::<F, 3, 2>::new(8, 57);
    let leaves = hash_nullifier_pre_images(nullifier_tree_preimages);
    let tree = MerkleTree::new(&mut hash, leaves).unwrap();
    tree.get_root()
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
pub fn update_nullifier_tree<F: BigPrimeField>(
    nullifier_old_preimages: Vec<IndexedMerkleTreeLeaf<F>>,
    new_val: F,
    new_val_idx: u64,
) -> (Vec<IndexedMerkleTreeLeaf<F>>, IndexedMerkleTreeInput<F>) {
    let mut nullifier_tree_preimages = nullifier_old_preimages.clone();
    let mut hasher = Poseidon::<F, 3, 2>::new(8, 57);
    let old_tree = MerkleTree::new(
        &mut hasher,
        hash_nullifier_pre_images(nullifier_old_preimages.clone()),
    )
    .unwrap();
    let old_root = old_tree.get_root();

    let mut low_leaf_idx = 0;
    for (i, node) in nullifier_old_preimages.iter().enumerate() {
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
        }
    }
    let low_leaf = nullifier_old_preimages[low_leaf_idx].clone();
    let (low_leaf_proof, low_leaf_proof_helper) = old_tree.get_proof(low_leaf_idx);
    let mut hasher = Poseidon::<F, 3, 2>::new(8, 57);
    let new_tree = MerkleTree::new(
        &mut hasher,
        hash_nullifier_pre_images(nullifier_tree_preimages.clone()),
    )
    .unwrap();
    let new_root = new_tree.get_root();
    let new_leaf = nullifier_tree_preimages[new_val_idx as usize].clone();
    let (new_leaf_proof, new_leaf_proof_helper) = new_tree.get_proof(new_val_idx as usize);
    let new_leaf_index = F::from(new_val_idx as u64);
    let is_new_leaf_largest = if new_leaf.next_val == F::ZERO {
        F::from(1u64)
    } else {
        F::from(0u64)
    };

    let input = IndexedMerkleTreeInput::new(
        old_root,
        low_leaf,
        low_leaf_proof,
        low_leaf_proof_helper,
        new_root,
        new_leaf,
        new_leaf_index,
        new_leaf_proof,
        new_leaf_proof_helper,
    );
    (nullifier_tree_preimages, input)
}
