use context_interface::result::Output;
use core::ops::Range;
use interpreter::{CallOutcome, CreateOutcome, Gas, InstructionResult, InterpreterResult};
use primitives::Address;

/// Call Frame
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CallFrame {
    /// Call frame has return memory range where output will be stored.
    pub return_memory_range: Range<usize>,
}

/// Create Frame
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CreateFrame {
    /// Create frame has a created address.
    pub created_address: Address,
}

/// Eof Create Frame
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EOFCreateFrame {
    pub created_address: Address,
}

/// Frame Data
///
/// [`FrameData`] bundles different types of frames.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FrameData {
    Call(CallFrame),
    Create(CreateFrame),
    EOFCreate(EOFCreateFrame),
}

/// Frame Result
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
pub enum FrameResult {
    Call(CallOutcome),
    Create(CreateOutcome),
    EOFCreate(CreateOutcome),
}

macro_rules! get_interpreter_result {
    ($self:expr) => {
        match $self {
            FrameResult::Call(CallOutcome { result, .. })
            | FrameResult::EOFCreate(CreateOutcome { result, .. })
            | FrameResult::Create(CreateOutcome { result, .. }) => result,
        }
    };
}

impl FrameResult {
    /// Casts frame result to interpreter result.
    #[inline]
    pub fn into_interpreter_result(self) -> InterpreterResult {
        get_interpreter_result!(self)
    }

    /// Returns reference to interpreter result.
    #[inline]
    pub fn interpreter_result(&self) -> &InterpreterResult {
        get_interpreter_result!(self)
    }

    /// Returns mutable reference to interpreter result.
    #[inline]
    pub fn interpreter_result_mut(&mut self) -> &mut InterpreterResult {
        get_interpreter_result!(self)
    }

    /// Returns execution output.
    pub fn output(&self) -> Output {
        let output = get_interpreter_result!(self).output.clone();

        if let FrameResult::Create(outcome) | FrameResult::EOFCreate(outcome) = self {
            return Output::Create(output, outcome.address);
        }

        Output::Call(output)
    }

    /// Returns reference to gas.
    #[inline]
    pub fn gas(&self) -> &Gas {
        &get_interpreter_result!(self).gas
    }

    /// Returns mutable reference to interpreter result.
    #[inline]
    pub fn gas_mut(&mut self) -> &mut Gas {
        &mut get_interpreter_result!(self).gas
    }

    /// Return Instruction result.
    #[inline]
    pub fn instruction_result(&self) -> InstructionResult {
        self.interpreter_result().result
    }
}

impl FrameData {
    pub fn new_create(created_address: Address) -> Self {
        Self::Create(CreateFrame { created_address })
    }

    pub fn new_call(return_memory_range: Range<usize>) -> Self {
        Self::Call(CallFrame {
            return_memory_range,
        })
    }

    /// Returns true if frame is call frame.
    pub fn is_call(&self) -> bool {
        matches!(self, Self::Call { .. })
    }

    /// Returns true if frame is create frame.
    pub fn is_create(&self) -> bool {
        matches!(self, Self::Create { .. })
    }

    /// Returns created address if frame is create otherwise returns None.
    pub fn created_address(&self) -> Option<Address> {
        match self {
            Self::Create(create_frame) => Some(create_frame.created_address),
            _ => None,
        }
    }
}
