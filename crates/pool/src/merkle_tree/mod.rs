
#[cfg(test)]
mod tests;
use core::{fmt, fmt::Display};

pub struct VirtualMerkleTree<N> {
    // NP TODO doc
    /// NP TODO doc
    nodes: Vec<Vec<N>>,

    // NP TODO put height as a field or generic? Otherwise optimise calls
}

pub trait Compressor<T> {
    fn compress(left: &T, right: &T) -> T;
    fn iterated_compression(n: usize) -> T;
        // let precomputed_hashes = successors(
        //     Some(empty_leaf),
        //     |prev_hash| C::compress(prev_hash, prev_hash)
        // );
        // let num_leaves = 1 << H - 1;

        // let mut nodes = Vec::with_capacity(num_leaves);

        // for i in ..H {
        //     let
        // } 
}



impl <N: Clone + PartialEq> VirtualMerkleTree<N> 
{
    pub fn empty(height: usize) -> Self {
        VirtualMerkleTree {
            nodes: vec![vec![]; height + 1]
        }
    }

    pub fn num_leaves(&self) -> usize {
        self.leaves().len()
    }

    pub fn nodes(&self, level: usize) -> &[N] {
        self.nodes.get(level).unwrap_or_else(||
            panic!(
                "Requested level {}, but the tree only has {} levels",
                level,
                self.nodes.len()
            )
        )
    }

    pub fn leaves(&self) -> &Vec<N> {
        &self.nodes.last().unwrap()
    }

    pub fn is_full(&self) -> bool {
        self.leaves().len() == 1 << self.height()
    }

    pub fn height(&self) -> usize {
        self.nodes.len() - 1
    }
    
    pub fn insert<C: Compressor<N>>(&mut self, leaf: N) {

        // NP TODO rethink whether in loop one can work with references/optimise
        // memory management

        let num_leaves = self.num_leaves();
        
        // NP TODO this calls num_leaves, make sure things are kind of optimal
        assert!(!self.is_full(), "Cannot insert a leaf into a full tree");
        
        let mut index = num_leaves;
        let mut node = leaf;
        let mut is_left_child = index % 2 == 0;

        self.nodes.last_mut().unwrap().push(node.clone());

        for level in (1..=self.height()).rev() {
            let (left_child, right_child) = if is_left_child {
                (node, self.virtual_node_unchecked::<C>(level, index + 1)) 
            } else { 
                (self.virtual_node_unchecked::<C>(level, index - 1), node)
            };
                        
            node = C::compress(&left_child, &right_child);
            index = index / 2;
            
            // If the node with index `index` is already present, update it.
            // Otherwise, set it to the new value.
            let level_nodes = &mut self.nodes[level - 1];
            
            if level_nodes.len() == index {
                level_nodes.push(node.clone());
            } else {
                level_nodes[index] = node.clone();
            }

            is_left_child = index % 2 == 0;
        }
    }

    pub fn virtual_leaf<C: Compressor<N>>(&self, index: usize) -> N {
        let leaves = self.leaves();
        
        assert!(
            index < leaves.len(),
            "Requested leaf index {} is out of bounds for a tree with height {}",
            index,
            self.height()
        );
        
        leaves.get(index).cloned().unwrap_or_else(|| C::iterated_compression(0))
    }

    pub fn leaf(&self, index: usize) -> N {
        self.leaves().get(index).cloned().unwrap_or_else(
            || panic!(
                "Requested leaf with index {}, but the tree only has {} leaves",
                index,
                self.num_leaves()
            )
        )
    }

    pub fn virtual_node<C: Compressor<N>>(&self, level: usize, index: usize) -> N {

        let height = self.height();

        assert!(
            level <= height,
            "Requested level {}, but the tree only has {} levels",
            level,
            height
        );

        assert!(index < 1 << level,
            "Requested index {} is out of bounds for level {}",
            index,
            level
        );

        self.nodes[level].get(index).cloned().unwrap_or_else(|| C::iterated_compression(height - level))
    }

    fn virtual_node_unchecked<C: Compressor<N>>(&self, level: usize, index: usize) -> N {
        self.nodes[level].get(index).cloned().unwrap_or_else(|| C::iterated_compression(self.height() - level))
    }

    pub fn path(&self, mut index: usize) -> Vec<N> {
        assert!(
            index < self.num_leaves(),
            "Requested path from leaf with index {}, but the tree only has {} leaves",
            index,
            self.num_leaves()
        );

        self.nodes.iter().rev().map(|level| {
            let node = level[index].clone();
            index = index / 2;
            node
        }).collect()
    }

    pub fn path_siblings<C: Compressor<N>>(&self, mut index: usize) -> Vec<N> {
        assert!(
            index < self.num_leaves(),
            "Requested path siblings from leaf with index {}, but the tree only has {} leaves",
            index,
            self.num_leaves()
        );

        (1..=self.height()).rev().map(|level| {
            let sibling_index = index + 1 - 2 * (index % 2);
            let sibling = self.virtual_node_unchecked::<C>(level, sibling_index).clone();
            index = index / 2;
            sibling
        }).collect()
    }

    pub fn verify_leaf<C: Compressor<N>>(leaf: N, root: &N, index: usize, path_siblings: &[N]) -> bool {
        
        let (mut leaf, mut index) = (leaf, index);
        for sibling in path_siblings {

            leaf = if index % 2 == 0 {
                C::compress(&leaf, sibling)
            } else {
                C::compress(sibling, &leaf)
            };

            index = index / 2;
        }

        leaf == *root
    }

    fn root<C: Compressor<N>>(&self) -> N {
        self.virtual_node_unchecked::<C>(0, 0)
    }
}

// NP TODO remove or improve, eg print empty
impl<N: Clone + Display> Display for VirtualMerkleTree<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for level in self.nodes.iter() {
            writeln!(f, "{:?}", level.iter().map(|n| n.to_string()).collect::<Vec<String>>().join(", "))?;
        }
        Ok(())
    }
}
