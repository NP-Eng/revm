
pub struct VirtualMerkleTree<L, C: Compressor<L>> {
    // NP TODO doc
    /// NP TODO doc
    nodes: Vec<L>;
    empty_nodes
}

pub trait Compressor<T> {
    fn compress(left: &T, right: &T) -> T;
}

impl<H, L, C> MerkleTree<H, L, C> 
where
    const H: usize,
    C: Compressor<L>
{
    pub fn empty(empty_leaf: &L) -> Self {
        let precomputed_hashes = successors(
            Some(empty_leaf),
            |prev_hash| C::compress(prev_hash, prev_hash)
        );
        let num_leaves = 1 << H - 1;

        let mut nodes = Vec::with_capacity(num_leaves);

        for i in ..H {
            let
        } 
    }

    pub fn num_leaves(&self) -> usize {
        self.leaves().len();
    };

    pub fn is_full(&self) -> bool;

    pub fn leaves(&self) -> &[L];
    
    pub fn insert(&mut self, &L);

    pub fn leaf(&self, index: usize) -> &L {
        // Option A: leaves first
        assert!(index < self.num_leaves());
        &self.nodes[index]
    };

    fn virtual_leaf(&self, index: usize) -> &L {
        if index < self.num_leaves() {
            self.leaves.get(index).unwrap()
        } else {
            L::defaut()
        }
    }

    pub fn node(&self, layer, index: usize) -> &L {
    }
}


