use rand::Rng;

use crate::merkle_tree::{Compressor, VirtualMerkleTree};

impl Compressor<isize> for () {
    fn compress(left: &isize, right: &isize) -> isize {
        left * right
    }

    fn iterated_compression(n: usize) -> isize {
        if n == 0 {
            // Default value
            2
        } else {
            let prev = Self::iterated_compression(n - 1);
            Self::compress(&prev, &prev)
        }
    }
}

struct AdditiveCompressor<const S: isize> {}

impl<const S: isize> Compressor<isize> for AdditiveCompressor<S> {
    fn compress(left: &isize, right: &isize) -> isize {
        left + right - S
    }

    fn iterated_compression(n: usize) -> isize {
        if n == 0 {
            // Default value
            3
        } else {
            let prev = Self::iterated_compression(n - 1);
            Self::compress(&prev, &prev)
        }
    }
}

#[test]
fn test_root() {
    let height = 4;
    
    let mut rng = rand::thread_rng();
    let mut tree = VirtualMerkleTree::empty(height);

    let mut expected_root = 1 << (1 << height);
    assert_eq!(tree.root::<()>(), expected_root);

    for _ in 1..=(1 << height) {
        let mut new_leaf = 0;
        
        while new_leaf == 0 {
            new_leaf = rng.gen_range(-10..=10);
        }

        tree.insert::<()>(new_leaf);

        expected_root = expected_root / 2 * new_leaf;

        assert_eq!(tree.root::<()>(), expected_root);
    }
}

#[test]
fn test_path_siblings() {
    let height = 4;
    
    let mut rng = rand::thread_rng();
    let mut tree = VirtualMerkleTree::empty(height);

    (0..(1 << (height - 1))).for_each(|_| {
        let val: isize = rng.gen();
        tree.insert::<AdditiveCompressor<5>>(val / (1 << (height + 1)))
    });

    // NP TODO this is really ugly; Maybe treat the empty-tree case separately?
    let root = tree.root::<AdditiveCompressor<5>>();

    for (i, leaf) in tree.leaves().iter().enumerate() {
        let sibling_path = tree.path_siblings::<AdditiveCompressor<5>>(i);
        assert!(VirtualMerkleTree::<isize>::verify_leaf::<AdditiveCompressor<5>>(*leaf, &root, i, &sibling_path));
    }
}


