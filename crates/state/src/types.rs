use super::{Account, EvmStorageSlot};
use pool::NoteTree;
use primitives::{Address, HashMap, U256};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// EVM State is a mapping from addresses to accounts and a note tree
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EvmState {
    pub account_state: HashMap<Address, Account>,
    pub note_tree: NoteTree,
}

/// Structure used for EIP-1153 transient storage
pub type TransientStorage = HashMap<(Address, U256), U256>;

/// An account's Storage is a mapping from 256-bit integer keys to [EvmStorageSlot]s.
pub type EvmStorage = HashMap<U256, EvmStorageSlot>;

impl EvmState {
    pub fn clear(&mut self) {
        self.account_state.clear();
        self.note_tree.clear();
    }
}
