use rand::Rng;

use crate::merkle_tree::{note_tree::KeccakCompressor, Compressor, VirtualMerkleTree};

const MULTIPLICATIVE_COMPRESSOR_ITERATED_HASHES: [isize; 5] = [2, 4, 16, 256, 65536];
const ADDMINUS5_COMPRESSOR_ITERATED_HASHES: [isize; 5] = [3, 1, -3, -11, -27];

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
