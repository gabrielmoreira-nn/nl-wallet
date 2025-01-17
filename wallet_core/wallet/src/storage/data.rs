use serde::{de::DeserializeOwned, Deserialize, Serialize};

use wallet_common::account::{messages::auth::WalletCertificate, serialization::Base64Bytes};

pub trait KeyedData: Serialize + DeserializeOwned {
    const KEY: &'static str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationData {
    pub pin_salt: Base64Bytes,
    pub wallet_certificate: WalletCertificate,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InstructionData {
    pub instruction_sequence_number: u64,
}

impl KeyedData for RegistrationData {
    const KEY: &'static str = "registration";
}

impl KeyedData for InstructionData {
    const KEY: &'static str = "instructions";
}
