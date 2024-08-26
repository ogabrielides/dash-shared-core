# [allow (clippy :: let_and_return , clippy :: suspicious_else_formatting , clippy :: redundant_field_names , dead_code , redundant_semicolons , unused_braces , unused_imports , unused_unsafe , unused_variables , unused_qualifications)] pub mod types { pub mod messages { pub mod send_coinjoin_queue { # [doc = "FFI-representation of the # [doc = \"FFI-representation of the crate :: messages :: send_coinjoin_queue :: SendCoinJoinQueue\"]"] # [repr (C)] # [derive (Clone)] # [allow (non_camel_case_types)] pub struct SendCoinJoinQueue { pub send : bool , } impl ferment_interfaces :: FFIConversion < crate :: messages :: send_coinjoin_queue :: SendCoinJoinQueue > for SendCoinJoinQueue { unsafe fn ffi_from_const (ffi : * const SendCoinJoinQueue) -> crate :: messages :: send_coinjoin_queue :: SendCoinJoinQueue { let ffi_ref = & * ffi ; crate :: messages :: send_coinjoin_queue :: SendCoinJoinQueue { send : ffi_ref . send , } } unsafe fn ffi_to_const (obj : crate :: messages :: send_coinjoin_queue :: SendCoinJoinQueue) -> * const SendCoinJoinQueue { ferment_interfaces :: boxed (SendCoinJoinQueue { send : obj . send , }) } unsafe fn destroy (ffi : * mut SendCoinJoinQueue) { ferment_interfaces :: unbox_any (ffi) ; } } impl Drop for SendCoinJoinQueue { fn drop (& mut self) { unsafe { let ffi_ref = self ; ; } } } # [doc = r" # Safety"] # [allow (non_snake_case)] # [no_mangle] pub unsafe extern "C" fn SendCoinJoinQueue_ctor (send : bool) -> * mut SendCoinJoinQueue { ferment_interfaces :: boxed (SendCoinJoinQueue { send , }) } # [doc = r" # Safety"] # [allow (non_snake_case)] # [no_mangle] pub unsafe extern "C" fn SendCoinJoinQueue_destroy (ffi : * mut SendCoinJoinQueue) { ferment_interfaces :: unbox_any (ffi) ; } } pub mod coinjoin_complete_message { # [doc = "FFI-representation of the # [doc = \"FFI-representation of the crate :: messages :: coinjoin_complete_message :: CoinJoinCompleteMessage\"]"] # [repr (C)] # [derive (Clone)] # [allow (non_camel_case_types)] pub struct CoinJoinCompleteMessage { pub msg_session_id : i32 , pub msg_message_id : * mut crate :: fermented :: types :: messages :: pool_message :: PoolMessage , } impl ferment_interfaces :: FFIConversion < crate :: messages :: coinjoin_complete_message :: CoinJoinCompleteMessage > for CoinJoinCompleteMessage { unsafe fn ffi_from_const (ffi : * const CoinJoinCompleteMessage) -> crate :: messages :: coinjoin_complete_message :: CoinJoinCompleteMessage { let ffi_ref = & * ffi ; crate :: messages :: coinjoin_complete_message :: CoinJoinCompleteMessage { msg_session_id : ffi_ref . msg_session_id , msg_message_id : ferment_interfaces :: FFIConversion :: ffi_from (ffi_ref . msg_message_id) , } } unsafe fn ffi_to_const (obj : crate :: messages :: coinjoin_complete_message :: CoinJoinCompleteMessage) -> * const CoinJoinCompleteMessage { ferment_interfaces :: boxed (CoinJoinCompleteMessage { msg_session_id : obj . msg_session_id , msg_message_id : ferment_interfaces :: FFIConversion :: ffi_to (obj . msg_message_id) , }) } unsafe fn destroy (ffi : * mut CoinJoinCompleteMessage) { ferment_interfaces :: unbox_any (ffi) ; } } impl Drop for CoinJoinCompleteMessage { fn drop (& mut self) { unsafe { let ffi_ref = self ; ; ferment_interfaces :: unbox_any (ffi_ref . msg_message_id) ; ; } } } # [doc = r" # Safety"] # [allow (non_snake_case)] # [no_mangle] pub unsafe extern "C" fn CoinJoinCompleteMessage_ctor (msg_session_id : i32 , msg_message_id : * mut crate :: fermented :: types :: messages :: pool_message :: PoolMessage) -> * mut CoinJoinCompleteMessage { ferment_interfaces :: boxed (CoinJoinCompleteMessage { msg_session_id , msg_message_id , }) } # [doc = r" # Safety"] # [allow (non_snake_case)] # [no_mangle] pub unsafe extern "C" fn CoinJoinCompleteMessage_destroy (ffi : * mut CoinJoinCompleteMessage) { ferment_interfaces :: unbox_any (ffi) ; } } pub mod coinjoin_accept_message { # [doc = "FFI-representation of the # [doc = \"FFI-representation of the crate :: messages :: coinjoin_accept_message :: CoinJoinAcceptMessage\"]"] # [repr (C)] # [derive (Clone)] # [allow (non_camel_case_types)] pub struct CoinJoinAcceptMessage { pub denomination : u32 , pub tx_collateral : * mut dash_spv_masternode_processor :: tx :: transaction :: Transaction , } impl ferment_interfaces :: FFIConversion < crate :: messages :: coinjoin_accept_message :: CoinJoinAcceptMessage > for CoinJoinAcceptMessage { unsafe fn ffi_from_const (ffi : * const CoinJoinAcceptMessage) -> crate :: messages :: coinjoin_accept_message :: CoinJoinAcceptMessage { let ffi_ref = & * ffi ; crate :: messages :: coinjoin_accept_message :: CoinJoinAcceptMessage { denomination : ffi_ref . denomination , tx_collateral : ferment_interfaces :: FFIConversion :: ffi_from (ffi_ref . tx_collateral) , } } unsafe fn ffi_to_const (obj : crate :: messages :: coinjoin_accept_message :: CoinJoinAcceptMessage) -> * const CoinJoinAcceptMessage { ferment_interfaces :: boxed (CoinJoinAcceptMessage { denomination : obj . denomination , tx_collateral : ferment_interfaces :: FFIConversion :: ffi_to (obj . tx_collateral) , }) } unsafe fn destroy (ffi : * mut CoinJoinAcceptMessage) { ferment_interfaces :: unbox_any (ffi) ; } } impl Drop for CoinJoinAcceptMessage { fn drop (& mut self) { unsafe { let ffi_ref = self ; ; ferment_interfaces :: unbox_any (ffi_ref . tx_collateral) ; ; } } } # [doc = r" # Safety"] # [allow (non_snake_case)] # [no_mangle] pub unsafe extern "C" fn CoinJoinAcceptMessage_ctor (denomination : u32 , tx_collateral : * mut dash_spv_masternode_processor :: tx :: transaction :: Transaction) -> * mut CoinJoinAcceptMessage { ferment_interfaces :: boxed (CoinJoinAcceptMessage { denomination , tx_collateral , }) } # [doc = r" # Safety"] # [allow (non_snake_case)] # [no_mangle] pub unsafe extern "C" fn CoinJoinAcceptMessage_destroy (ffi : * mut CoinJoinAcceptMessage) { ferment_interfaces :: unbox_any (ffi) ; } } pub mod coinjoin_final_transaction { # [doc = "FFI-representation of the # [doc = \"FFI-representation of the crate :: messages :: coinjoin_final_transaction :: CoinJoinFinalTransaction\"]"] # [repr (C)] # [derive (Clone)] # [allow (non_camel_case_types)] pub struct CoinJoinFinalTransaction { pub msg_session_id : i32 , pub tx : * mut dash_spv_masternode_processor :: tx :: transaction :: Transaction , } impl ferment_interfaces :: FFIConversion < crate :: messages :: coinjoin_final_transaction :: CoinJoinFinalTransaction > for CoinJoinFinalTransaction { unsafe fn ffi_from_const (ffi : * const CoinJoinFinalTransaction) -> crate :: messages :: coinjoin_final_transaction :: CoinJoinFinalTransaction { let ffi_ref = & * ffi ; crate :: messages :: coinjoin_final_transaction :: CoinJoinFinalTransaction { msg_session_id : ffi_ref . msg_session_id , tx : ferment_interfaces :: FFIConversion :: ffi_from (ffi_ref . tx) , } } unsafe fn ffi_to_const (obj : crate :: messages :: coinjoin_final_transaction :: CoinJoinFinalTransaction) -> * const CoinJoinFinalTransaction { ferment_interfaces :: boxed (CoinJoinFinalTransaction { msg_session_id : obj . msg_session_id , tx : ferment_interfaces :: FFIConversion :: ffi_to (obj . tx) , }) } unsafe fn destroy (ffi : * mut CoinJoinFinalTransaction) { ferment_interfaces :: unbox_any (ffi) ; } } impl Drop for CoinJoinFinalTransaction { fn drop (& mut self) { unsafe { let ffi_ref = self ; ; ferment_interfaces :: unbox_any (ffi_ref . tx) ; ; } } } # [doc = r" # Safety"] # [allow (non_snake_case)] # [no_mangle] pub unsafe extern "C" fn CoinJoinFinalTransaction_ctor (msg_session_id : i32 , tx : * mut dash_spv_masternode_processor :: tx :: transaction :: Transaction) -> * mut CoinJoinFinalTransaction { ferment_interfaces :: boxed (CoinJoinFinalTransaction { msg_session_id , tx , }) } # [doc = r" # Safety"] # [allow (non_snake_case)] # [no_mangle] pub unsafe extern "C" fn CoinJoinFinalTransaction_destroy (ffi : * mut CoinJoinFinalTransaction) { ferment_interfaces :: unbox_any (ffi) ; } } pub mod coinjoin_status_update { # [doc = "FFI-representation of the # [doc = \"FFI-representation of the crate :: messages :: coinjoin_status_update :: CoinJoinStatusUpdate\"]"] # [repr (C)] # [derive (Clone)] # [allow (non_camel_case_types)] pub struct CoinJoinStatusUpdate { pub session_id : i32 , pub pool_state : * mut crate :: fermented :: types :: messages :: pool_state :: PoolState , pub status_update : * mut crate :: fermented :: types :: messages :: pool_status_update :: PoolStatusUpdate , pub message_id : * mut crate :: fermented :: types :: messages :: pool_message :: PoolMessage , } impl ferment_interfaces :: FFIConversion < crate :: messages :: coinjoin_status_update :: CoinJoinStatusUpdate > for CoinJoinStatusUpdate { unsafe fn ffi_from_const (ffi : * const CoinJoinStatusUpdate) -> crate :: messages :: coinjoin_status_update :: CoinJoinStatusUpdate { let ffi_ref = & * ffi ; crate :: messages :: coinjoin_status_update :: CoinJoinStatusUpdate { session_id : ffi_ref . session_id , pool_state : ferment_interfaces :: FFIConversion :: ffi_from (ffi_ref . pool_state) , status_update : ferment_interfaces :: FFIConversion :: ffi_from (ffi_ref . status_update) , message_id : ferment_interfaces :: FFIConversion :: ffi_from (ffi_ref . message_id) , } } unsafe fn ffi_to_const (obj : crate :: messages :: coinjoin_status_update :: CoinJoinStatusUpdate) -> * const CoinJoinStatusUpdate { ferment_interfaces :: boxed (CoinJoinStatusUpdate { session_id : obj . session_id , pool_state : ferment_interfaces :: FFIConversion :: ffi_to (obj . pool_state) , status_update : ferment_interfaces :: FFIConversion :: ffi_to (obj . status_update) , message_id : ferment_interfaces :: FFIConversion :: ffi_to (obj . message_id) , }) } unsafe fn destroy (ffi : * mut CoinJoinStatusUpdate) { ferment_interfaces :: unbox_any (ffi) ; } } impl Drop for CoinJoinStatusUpdate { fn drop (& mut self) { unsafe { let ffi_ref = self ; ; ferment_interfaces :: unbox_any (ffi_ref . pool_state) ; ; ferment_interfaces :: unbox_any (ffi_ref . status_update) ; ; ferment_interfaces :: unbox_any (ffi_ref . message_id) ; ; } } } # [doc = r" # Safety"] # [allow (non_snake_case)] # [no_mangle] pub unsafe extern "C" fn CoinJoinStatusUpdate_ctor (session_id : i32 , pool_state : * mut crate :: fermented :: types :: messages :: pool_state :: PoolState , status_update : * mut crate :: fermented :: types :: messages :: pool_status_update :: PoolStatusUpdate , message_id : * mut crate :: fermented :: types :: messages :: pool_message :: PoolMessage) -> * mut CoinJoinStatusUpdate { ferment_interfaces :: boxed (CoinJoinStatusUpdate { session_id , pool_state , status_update , message_id , }) } # [doc = r" # Safety"] # [allow (non_snake_case)] # [no_mangle] pub unsafe extern "C" fn CoinJoinStatusUpdate_destroy (ffi : * mut CoinJoinStatusUpdate) { ferment_interfaces :: unbox_any (ffi) ; } } } pub mod ffi { pub mod input_value { # [doc = "FFI-representation of the # [doc = \"FFI-representation of the crate :: ffi :: input_value :: InputValue\"]"] # [repr (C)] # [derive (Clone)] # [allow (non_camel_case_types)] pub struct InputValue { pub is_valid : bool , pub value : u64 , } impl ferment_interfaces :: FFIConversion < crate :: ffi :: input_value :: InputValue > for InputValue { unsafe fn ffi_from_const (ffi : * const InputValue) -> crate :: ffi :: input_value :: InputValue { let ffi_ref = & * ffi ; crate :: ffi :: input_value :: InputValue { is_valid : ffi_ref . is_valid , value : ffi_ref . value , } } unsafe fn ffi_to_const (obj : crate :: ffi :: input_value :: InputValue) -> * const InputValue { ferment_interfaces :: boxed (InputValue { is_valid : obj . is_valid , value : obj . value , }) } unsafe fn destroy (ffi : * mut InputValue) { ferment_interfaces :: unbox_any (ffi) ; } } impl Drop for InputValue { fn drop (& mut self) { unsafe { let ffi_ref = self ; ; ; } } } # [doc = r" # Safety"] # [allow (non_snake_case)] # [no_mangle] pub unsafe extern "C" fn InputValue_ctor (is_valid : bool , value : u64) -> * mut InputValue { ferment_interfaces :: boxed (InputValue { is_valid , value , }) } # [doc = r" # Safety"] # [allow (non_snake_case)] # [no_mangle] pub unsafe extern "C" fn InputValue_destroy (ffi : * mut InputValue) { ferment_interfaces :: unbox_any (ffi) ; } } } } # [allow (clippy :: let_and_return , clippy :: suspicious_else_formatting , clippy :: redundant_field_names , dead_code , redundant_semicolons , unused_braces , unused_imports , unused_unsafe , unused_variables , unused_qualifications)] pub mod generics { }