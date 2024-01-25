use std::ffi::c_void;
use dash_spv_masternode_processor::types;

use crate::models::InputValue;

pub type GetInputValueByPrevoutHash = unsafe extern "C" fn(
    prevout_hash: *mut [u8; 32],
    index: u32,
    context: *const c_void,
) -> *mut InputValue;

pub type HasChainLock = unsafe extern "C" fn(
    block: *mut types::Block,
    context: *const c_void,
) -> bool;

pub type DestroyInputValue = unsafe extern "C" fn(
    input_value: *mut InputValue,
);

pub type GetWalletTransaction = unsafe extern "C" fn(
    hash: *mut [u8; 32],
    context: *const c_void,
) -> *mut types::Transaction;

pub type DestroyWalletTransaction = unsafe extern "C" fn(
    input_value: *mut types::Transaction,
);

pub type IsMineInput = unsafe extern "C" fn(
    prevout_hash: *mut [u8; 32],
    index: u32,
    context: *const c_void,
) -> bool;

pub type IsMineAddress = unsafe extern "C" fn(
    address: *mut [u8; 32],
    context: *const c_void,
) -> bool;