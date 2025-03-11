// NP TODO doc

mod merkle_tree;

pub const NOTE_TREE_HEIGHT: usize = 20;

pub use merkle_tree::note_tree::{NoteTree, KeccakCompressor};
