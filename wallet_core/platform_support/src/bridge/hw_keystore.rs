use std::fmt::Debug;

use super::get_bridge_collection;

pub use crate::hw_keystore::KeyStoreError;

// this is required to catch UnexpectedUniFFICallbackError
impl From<uniffi::UnexpectedUniFFICallbackError> for KeyStoreError {
    fn from(value: uniffi::UnexpectedUniFFICallbackError) -> Self {
        Self::BridgingError { reason: value.reason }
    }
}

// the callback traits defined in the UDL, which we have write out here ourselves
pub trait SigningKeyBridge: Send + Sync + Debug {
    fn public_key(&self, identifier: String) -> Result<Vec<u8>, KeyStoreError>;
    fn sign(&self, identifier: String, payload: Vec<u8>) -> Result<Vec<u8>, KeyStoreError>;
}

pub trait EncryptionKeyBridge: Send + Sync + Debug {
    fn encrypt(&self, identifier: String, payload: Vec<u8>) -> Result<Vec<u8>, KeyStoreError>;
    fn decrypt(&self, identifier: String, payload: Vec<u8>) -> Result<Vec<u8>, KeyStoreError>;
}

pub fn get_signing_key_bridge() -> &'static dyn SigningKeyBridge {
    get_bridge_collection().signing_key.as_ref()
}

pub fn get_encryption_key_bridge() -> &'static dyn EncryptionKeyBridge {
    get_bridge_collection().encryption_key.as_ref()
}
