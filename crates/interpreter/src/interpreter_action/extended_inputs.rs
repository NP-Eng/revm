use alloy_consensus::transaction::CommitmentBytes;
use primitives::{Address, U256};

/// Inputs for a create call
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExtendedInputs {
    /// Caller address of the EVM
    pub caller: Address,
    /// The value to transfer
    pub value: U256,
    // NP TODO doc
    /// NP todo com
    pub commitment: CommitmentBytes,
}
