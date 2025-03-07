// NP TODO doc

mod merkle_tree;

pub const NOTE_TREE_HEIGHT: usize = 29;
pub type NoteTree = MerkleTree<NOTE_TREE_HEIGHT>;

pub use NoteTree;
