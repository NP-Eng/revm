use super::frame_data::FrameResult;
use bytecode::EOF_MAGIC_BYTES;
use context_interface::ContextTr;
use context_interface::Transaction;
use interpreter::ExtendedInputs;
use interpreter::{
    CallInputs, CallScheme, CallValue, CreateInputs, CreateScheme, EOFCreateInputs, EOFCreateKind,
    FrameInput, Gas,
};
use primitives::TxKind;
use specification::hardfork::SpecId;
use std::boxed::Box;

pub fn create_init_frame(tx: &impl Transaction, spec: SpecId, gas_limit: u64) -> FrameInput {
    // Make new frame action.
    let caller = tx.caller();
    let value = tx.value();

    match tx.kind() {
        TxKind::Call(target_address) => FrameInput::Call(Box::new(CallInputs {
            input: tx.input().clone(),
            gas_limit,
            target_address,
            bytecode_address: target_address,
            caller,
            value: CallValue::Transfer(value),
            scheme: CallScheme::Call,
            is_static: false,
            is_eof: false,
            return_memory_offset: 0..0,
        })),
        TxKind::Create => {
            if let Some(commitment) = tx.commitment() {
                FrameInput::Extended(ExtendedInputs {
                    caller,
                    value,
                    commitment: commitment.clone(),
                })
            } else {
                let input = tx.input().clone();

                // If first byte of data is magic 0xEF00, then it is EOFCreate.
                if spec.is_enabled_in(SpecId::OSAKA) && input.starts_with(&EOF_MAGIC_BYTES) {
                    FrameInput::EOFCreate(Box::new(EOFCreateInputs::new(
                        caller,
                        value,
                        gas_limit,
                        EOFCreateKind::Tx { initdata: input },
                    )))
                } else {
                    FrameInput::Create(Box::new(CreateInputs {
                        caller,
                        scheme: CreateScheme::Create,
                        value,
                        init_code: input,
                        gas_limit,
                    }))
                }
            }
        }
    }
}

/// TODO : Frame result should be a generic trait with needed functions.
pub fn last_frame_result<CTX: ContextTr>(context: CTX, frame_result: &mut FrameResult) {
    let instruction_result = frame_result.interpreter_result().result;
    let gas = frame_result.gas_mut();
    let remaining = gas.remaining();
    let refunded = gas.refunded();

    // Spend the gas limit. Gas is reimbursed when the tx returns successfully.
    *gas = Gas::new_spent(context.tx().gas_limit());

    if instruction_result.is_ok_or_revert() {
        gas.erase_cost(remaining);
    }

    if instruction_result.is_ok() {
        gas.record_refund(refunded);
    }
}
