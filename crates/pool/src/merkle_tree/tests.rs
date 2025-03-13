use primitives::B256;
use rand::Rng;

use std::fmt::{Display, Debug};
use crate::merkle_tree::{note_tree::KeccakCompressor, Compressor, VirtualMerkleTree};

const MULTIPLICATIVE_COMPRESSOR_ITERATED_HASHES: [isize; 5] = [2, 4, 16, 256, 65536];
const ADDMINUS5_COMPRESSOR_ITERATED_HASHES: [isize; 7] = [3, 1, -3, -11, -27, -59, -163];

#[derive(Eq, PartialEq, Debug)]
struct MultiplicativeCompressor;

impl Compressor<isize> for MultiplicativeCompressor {
    fn compress(&self, left: &isize, right: &isize) -> isize {
        left * right
    }

    fn iterated_compression(n: usize) -> &'static isize {
        MULTIPLICATIVE_COMPRESSOR_ITERATED_HASHES.get(n).unwrap()
    }

    fn load() -> Self {
        MultiplicativeCompressor
    }
}

#[derive(Eq, PartialEq, Debug)]
struct AddMinus5Compressor;

impl Compressor<isize> for AddMinus5Compressor {
    fn compress(&self, left: &isize, right: &isize) -> isize {
        left + right - 5
    }

    fn iterated_compression(n: usize) -> &'static isize {
        ADDMINUS5_COMPRESSOR_ITERATED_HASHES.get(n).unwrap()
    }

    fn load() -> Self {
        AddMinus5Compressor
    }
}

#[test]
fn test_root() {
    let height = 4;

    let mut rng = rand::thread_rng();
    let mut tree = VirtualMerkleTree::<_, MultiplicativeCompressor>::empty(height);

    let mut expected_root = 1 << (1 << height);
    assert_eq!(tree.root(), expected_root);

    for _ in 1..=(1 << height) {
        let mut new_leaf = 0;

        while new_leaf == 0 {
            new_leaf = rng.gen_range(-10..=10);
        }

        tree.insert(new_leaf);

        expected_root = expected_root / 2 * new_leaf;

        assert_eq!(tree.root(), expected_root);
    }
}

#[test]
fn test_path_siblings() {
    let height = 4;

    let mut rng = rand::thread_rng();
    let mut tree: VirtualMerkleTree::<isize, AddMinus5Compressor> = VirtualMerkleTree::empty(height);

    (0..(1 << (height - 1))).for_each(|_| {
        let val: isize = rng.gen();
        tree.insert(val / (1 << (height + 1)))
    });

    // NP TODO this is really ugly; Maybe treat the empty-tree case separately?
    let root = tree.root();

    for (i, leaf) in tree.leaves().iter().enumerate() {
        let sibling_path = tree.path_siblings(i);
        assert!(VirtualMerkleTree::verify_leaf(
            *leaf,
            &root,
            i,
            &sibling_path,
            &AddMinus5Compressor
        ));
    }
}

fn test_iterated_hashes<T: PartialEq + 'static + Clone, C: Compressor<T>>(compressor: C, n: usize) {
    let mut hash: T = C::iterated_compression(0).clone();

    for i in 1..n {
        hash = compressor.compress(&hash, &hash);
        assert!(&hash == C::iterated_compression(i));
    }
}

#[test]
fn test_iterated_hashes_mul() {
    test_iterated_hashes(AddMinus5Compressor, 5);
}

#[test]
fn test_iterated_hashes_addminus5() {
    test_iterated_hashes(AddMinus5Compressor, 5);
}

#[test]
fn test_iterated_hashes_keccak() {
    test_iterated_hashes(KeccakCompressor, 42);
}


fn test_pop_with_compressor<T, C>(height: usize, n_trees: usize, mut gen_leaf: impl FnMut() -> T)
where 
    T: PartialEq + Eq + Display + Debug + 'static + Clone,
    C: Compressor<T> + Clone,       
{
    // for num_leaves in ((1 << (height - 1)) + 1)..= (1 << height) {
    //     for num_popped in 0..=num_leaves {

    for (num_leaves, num_popped) in [
        ((1 << 15), (1 << 13) - 3),
        ((1 << 15), (1 << 13) - 2),
        ((1 << 15), (1 << 13) - 1),
        ((1 << 15), (1 << 13) + 0),
        ((1 << 15), (1 << 13) + 1),
        ((1 << 15), (1 << 13) + 2),
        ((1 << 15), (1 << 13) + 3),
        ((1 << 15) - 3, (1 << 13) - 3),
        ((1 << 15) - 2, (1 << 13) - 2),
        ((1 << 15) - 1, (1 << 13) - 1),
        ((1 << 15) + 0, (1 << 13) + 0),
        ((1 << 15) + 1, (1 << 13) + 1),
        ((1 << 15) + 2, (1 << 13) + 2),
        ((1 << 15) + 3, (1 << 13) + 2),
        ((1 << 15), (1 << 12) - 3),
        ((1 << 15), (1 << 12) - 2),
        ((1 << 15), (1 << 12) - 1),
        ((1 << 15), (1 << 12) + 0),
        ((1 << 15), (1 << 12) + 1),
        ((1 << 15), (1 << 12) + 2),
        ((1 << 15), (1 << 12) + 3),
        ((1 << 15) - 3, (1 << 12) - 3),
        ((1 << 15) - 2, (1 << 12) - 2),
        ((1 << 15) - 1, (1 << 12) - 1),
        ((1 << 15) + 0, (1 << 12) + 0),
        ((1 << 15) + 1, (1 << 12) + 1),
        ((1 << 15) + 2, (1 << 12) + 2),
        ((1 << 15) + 3, (1 << 12) + 2),
        ] {
        println!("num_leaves: {}, num_popped: {}", num_leaves, num_popped);

        let leaves = (0..num_leaves).map(|_| gen_leaf()).collect::<Vec<_>>();

        // NP TODO reintroduce
        // let mut tree = VirtualMerkleTree::<T, C>::empty(height);
        let mut tree = VirtualMerkleTree::<T, C>::empty(if num_leaves <= 1 << 15 { 15 } else { 16 });

        for leaf in leaves.clone() {
            tree.insert(leaf);
        }

        let mut trees_a = (0..n_trees).map(|_| tree.clone()).collect::<Vec<_>>();
        let mut trees_c = (0..n_trees).map(|_| tree.clone()).collect::<Vec<_>>();

        // time execution
        let start = std::time::Instant::now();
        for tree_a in trees_a.iter_mut() {
            tree_a.pop_a(num_popped);
        }
        let duration = start.elapsed();
        println!("   pop_a time: {:?}, average: {:?}", duration, duration / n_trees as u32);
        
        let start = std::time::Instant::now();
        for tree_c in trees_c.iter_mut() {
            tree_c.pop_c(num_popped);
        }
        let duration = start.elapsed();
        println!("   pop_c time: {:?}, average: {:?}", duration, duration / n_trees as u32);

        // NP TODO reintroduce
        // let mut trimmed_tree = VirtualMerkleTree::<T, C>::empty(height);
        let mut trimmed_tree = VirtualMerkleTree::<T, C>::empty(if num_leaves <= 1 << 15 { 15 } else { 16 });

        for leaf in leaves.into_iter().take(num_leaves - num_popped) {
            trimmed_tree.insert(leaf);
        }

        assert_eq!(trees_a.last().unwrap().root(), trimmed_tree.root(), "tree_a failed");
        //assert_eq!(trees_c.last().unwrap().root(), trimmed_tree.root(), "tree_c failed");
    }
}

#[test]
fn test_pop_keccak() {

    // Needs 2^15 nodes * 2^5 B * 2^6 trees * 2 algorithms = 2^31 = 2 GB of memory
    let height = 15;
    let n_trees = 1 << 6;
    
    let mut rng = rand::thread_rng();
    let gen_leaf = || B256::from(rng.gen::<[u8; 32]>());
    
    test_pop_with_compressor::<B256, KeccakCompressor>(height, n_trees, gen_leaf);
}
