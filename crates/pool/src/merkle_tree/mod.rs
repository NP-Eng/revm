pub(crate) mod note_tree;

#[cfg(test)]
mod tests;

use core::{fmt, fmt::Display, fmt::Debug};

pub trait Compressor<T>: Eq {
    fn compress(&self, left: &T, right: &T) -> T;
    fn iterated_compression(n: usize) -> &'static T;
    fn load() -> Self;
}

#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VirtualMerkleTree<N, C: Compressor<N>> {
    // NP TODO doc
    /// NP TODO doc
    nodes: Vec<Vec<N>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    // NP TODO test
    #[cfg_attr(feature = "serde", serde(default = "C::load"))]
    compressor: C,
    // NP TODO put height as a field or generic? Otherwise optimise calls
}

impl<N, C> VirtualMerkleTree<N, C>
where
    // NP TODO remove debug
    N: Debug + Clone + Eq + 'static,
    C: Compressor<N>,
{
    pub fn empty(height: usize) -> Self {
        VirtualMerkleTree {
            nodes: vec![vec![]; height + 1],
            compressor: C::load(),
        }
    }

    pub fn num_leaves(&self) -> usize {
        self.leaves().len()
    }

    pub fn nodes(&self, level: usize) -> &[N] {
        self.nodes.get(level).unwrap_or_else(|| {
            panic!(
                "Requested level {}, but the tree only has {} levels",
                level,
                self.nodes.len()
            )
        })
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

    pub fn insert(&mut self, leaf: N) -> Option<usize> {
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
                (node, self.virtual_node_unchecked(level, index + 1))
            } else {
                (self.virtual_node_unchecked(level, index - 1), node)
            };

            node = self.compressor.compress(&left_child, &right_child);
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

        Some(num_leaves)
    }

    pub fn virtual_leaf(&self, index: usize) -> N {
        let leaves = self.leaves();

        assert!(
            index < leaves.len(),
            "Requested leaf index {} is out of bounds for a tree with height {}",
            index,
            self.height()
        );

        // leaves.get(index).unwrap_or_else(|| C::iterated_compression(0).cloned())
        C::iterated_compression(0).clone()
    }

    pub fn leaf(&self, index: usize) -> N {
        self.leaves().get(index).cloned().unwrap_or_else(|| {
            panic!(
                "Requested leaf with index {}, but the tree only has {} leaves",
                index,
                self.num_leaves()
            )
        })
    }

    pub fn virtual_node(&self, level: usize, index: usize) -> N {
        let height = self.height();

        assert!(
            level <= height,
            "Requested level {}, but the tree only has {} levels",
            level,
            height
        );

        assert!(
            index < 1 << level,
            "Requested index {} is out of bounds for level {}",
            index,
            level
        );

        // TODO here and below, consider whether this should return a reference
        self.nodes[level]
            .get(index)
            .unwrap_or_else(|| C::iterated_compression(height - level))
            .clone()
    }

    fn virtual_node_unchecked(&self, level: usize, index: usize) -> N {
        self.nodes[level]
            .get(index)
            .unwrap_or_else(|| C::iterated_compression(self.height() - level))
            .clone()
    }

    pub fn path(&self, mut index: usize) -> Vec<N> {
        assert!(
            index < self.num_leaves(),
            "Requested path from leaf with index {}, but the tree only has {} leaves",
            index,
            self.num_leaves()
        );

        self.nodes
            .iter()
            .rev()
            .map(|level| {
                let node = level[index].clone();
                index = index / 2;
                node
            })
            .collect()
    }

    pub fn path_siblings(&self, mut index: usize) -> Vec<N> {
        assert!(
            index < self.num_leaves(),
            "Requested path siblings from leaf with index {}, but the tree only has {} leaves",
            index,
            self.num_leaves()
        );

        (1..=self.height())
            .rev()
            .map(|level| {
                let sibling_index = index + 1 - 2 * (index % 2);
                let sibling = self.virtual_node_unchecked(level, sibling_index).clone();
                index = index / 2;
                sibling
            })
            .collect()
    }

    pub fn verify_leaf(
        leaf: N,
        root: &N,
        index: usize,
        path_siblings: &[N],
        compressor: &C,
    ) -> bool {
        let (mut leaf, mut index) = (leaf, index);
        for sibling in path_siblings {
            leaf = if index % 2 == 0 {
                compressor.compress(&leaf, sibling)
            } else {
                compressor.compress(sibling, &leaf)
            };

            index = index / 2;
        }

        leaf == *root
    }

    pub fn root(&self) -> N {
        self.virtual_node_unchecked(0, 0)
    }
    pub fn clear(&mut self) {
        self.nodes.iter_mut().for_each(Vec::clear);
    }

    pub fn last_leaf(&self) -> Option<&N> {
        self.leaves().last()
    }

    pub fn pop(&mut self, n: usize) {

        if n == 0 {
            return;
        }

        assert!(
            n <= self.num_leaves(),
            "Attempted to pop {} leaves, but the tree only has {}",
            n,
            self.num_leaves()
        );

        let height = self.height();

        // Store the left siblings of affected notes
        let mut remaining = self.num_leaves() - n;
        let mut modified = false;

        // This early termination simplifies some checks later on
        if remaining == 0 {
            self.nodes = vec![vec![]; height + 1];
            return;
        }

        let left_siblings: Vec<Option<N>> = self.nodes.iter().skip(1).rev().map(|level| {
            let left_sibling = if remaining % 2 == 0 {
                if modified {
                    Some(level[remaining - 2].clone())
                } else {
                    None
                }
            } else {
                modified = true;
                None
            };

            remaining = (remaining + 1) / 2;

            left_sibling
        }).collect();

        // Main loop: trim discarded nodes and recompute affected ones
        let mut remaining = self.num_leaves() - n;
        let mut modified = false;

        self.nodes.last_mut().unwrap().truncate(remaining);

        let mut recomputed = if remaining % 2 == 1 {
            Some(self.leaves().last().unwrap().clone())
        } else {
            None
        };

        for (i, level) in self.nodes.iter_mut().rev().skip(1).enumerate() {
            recomputed = if remaining % 2 == 1 || modified {
                modified = true;
                
                let (left, right) = if remaining % 2 == 1 {
                    (&recomputed.unwrap(), &C::iterated_compression(i).clone())
                } else {
                    (left_siblings[i].as_ref().unwrap(), &recomputed.unwrap())
                };

                Some(self.compressor.compress(left, right))
            } else {
                None
            };

            remaining = (remaining + 1) / 2;
            level.truncate(remaining);

            if let Some(node) = &recomputed {
                level[remaining - 1] = node.clone();
            } else if remaining % 2 == 1 {
                recomputed = level.last().cloned();
            }
        }
    }
}

// NP TODO remove or improve, eg print empty
impl<N: Clone + Display, C: Compressor<N>> Display for VirtualMerkleTree<N, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for level in self.nodes.iter() {
            writeln!(
                f,
                "{:?}",
                level
                    .iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            )?;
        }
        Ok(())
    }
}

impl<N: Clone + Debug + Eq + 'static, C: Compressor<N>> fmt::Debug for VirtualMerkleTree<N, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "VirtualMerkleTree of height {} with root {:?}, number of leaves: {}",
            self.height(),
            self.root(),
            self.num_leaves(),
        )
    }
}