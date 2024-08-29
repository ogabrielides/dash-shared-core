use std::os::raw::c_void;
use std::sync::Arc;
use dash_sdk::platform::DataContract;
use drive_proof_verifier::ContextProvider;
use drive_proof_verifier::error::ContextProviderError;
use platform_value::Identifier;
use crate::FFIThreadSafeContext;

#[derive(Clone)]
pub struct PlatformProvider {
    pub get_quorum_public_key: Arc<dyn Fn(*const c_void, u32, [u8; 32], u32) -> Result<[u8; 48], ContextProviderError> + Send + Sync>,
    pub get_data_contract: Arc<dyn Fn(*const c_void, Identifier) -> Result<Option<Arc<DataContract>>, ContextProviderError> + Send + Sync>,
    pub context: Arc<FFIThreadSafeContext>
}

impl PlatformProvider {
    pub fn new<
        QPK: Fn(*const c_void, u32, [u8; 32], u32) -> Result<[u8; 48], ContextProviderError> + Send + Sync + 'static,
        DC: Fn(*const c_void, Identifier) -> Result<Option<Arc<DataContract>>, ContextProviderError> + Send + Sync + 'static>(
        get_quorum_public_key: QPK,
        get_data_contract: DC,
        context: Arc<FFIThreadSafeContext>
    ) -> Self {
        Self {
            get_quorum_public_key: Arc::new(get_quorum_public_key),
            get_data_contract: Arc::new(get_data_contract),
            context
        }
    }
}

impl ContextProvider for PlatformProvider {
    fn get_quorum_public_key(&self, quorum_type: u32, quorum_hash: [u8; 32], core_chain_locked_height: u32) -> Result<[u8; 48], ContextProviderError> {
        (self.get_quorum_public_key)(self.context.get(), quorum_type, quorum_hash, core_chain_locked_height)
    }
    fn get_data_contract(&self, id: &Identifier) -> Result<Option<Arc<DataContract>>, ContextProviderError> {
        (self.get_data_contract)(self.context.get(), id.clone())
    }
}
