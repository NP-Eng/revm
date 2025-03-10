// NP TODO doc

use merkle_tree::VirtualMerkleTree;

mod merkle_tree;

pub const NOTE_TREE_HEIGHT: usize = 29;
pub type NoteTree<N> = VirtualMerkleTree<N>; // NP TODO Compressor
