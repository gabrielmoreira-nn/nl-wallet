//! Data structures used in disclosure for everything that has to be signed with the mdoc's private key.
//! Mainly [`DeviceAuthentication`] and all data structures inside it, which includes a transcript
//! of the session so far.
//!
//! NB. "Device authentication" is not to be confused with the [`DeviceAuth`] data structure in the
//! [`disclosure`](super::disclosure) module (which contains the holder's signature over [`DeviceAuthentication`]
//! defined here).

use ciborium::value::Value;
use fieldnames_derive::FieldNames;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use serde_with::skip_serializing_none;
use std::fmt::Debug;

use crate::{
    iso::{disclosure::*, mdocs::*},
    utils::{
        cose::CoseKey,
        serialization::{
            CborIntMap, CborSeq, DeviceAuthenticationString, RequiredValue, RequiredValueTrait, TaggedBytes,
        },
    },
};

/// The data structure that the holder signs with the mdoc private key when disclosing attributes out of that mdoc.
/// Contains a.o. transcript of the session so far, acting as the challenge in a challenge-response mechanism,
/// and the "device-signed items" ([`DeviceNameSpaces`]): attributes that are signed only by the device, since they
/// are part of this data structure, but not by the issuer (i.e., self asserted attributes).
///
/// This data structure is computed by the holder and the RP during a session, and then signed and verified
/// respectively. It is not otherwise included in other data structures.
pub type DeviceAuthentication = CborSeq<DeviceAuthenticationKeyed>;

/// See [`DeviceAuthentication`].
pub type DeviceAuthenticationBytes = TaggedBytes<DeviceAuthentication>;

/// See [`DeviceAuthentication`].
#[derive(Serialize, Deserialize, FieldNames, Debug, Clone)]
pub struct DeviceAuthenticationKeyed {
    pub device_authentication: RequiredValue<DeviceAuthenticationString>,
    pub session_transcript: SessionTranscript,
    pub doc_type: DocType,
    pub device_name_spaces_bytes: DeviceNameSpacesBytes,
}

#[derive(Serialize, Deserialize, FieldNames, Debug, Clone)]
pub struct SessionTranscriptKeyed {
    pub device_engagement_bytes: DeviceEngagementBytes,
    pub ereader_key_bytes: ESenderKeyBytes,
    pub handover: Handover,
}

/// Transcript of the session so far. Used in [`DeviceAuthentication`].
pub type SessionTranscript = CborSeq<SessionTranscriptKeyed>;

pub type DeviceEngagementBytes = TaggedBytes<DeviceEngagement>;

#[derive(Debug, Clone)]
pub enum Handover {
    QRHandover,
    NFCHandover(NFCHandover),
}

#[derive(Debug, Clone)]
pub struct NFCHandover {
    pub handover_select_message: ByteBuf,
    pub handover_request_message: Option<ByteBuf>,
}

/// Describes available methods for the holder to connect to the RP.
pub type DeviceEngagement = CborIntMap<Engagement>;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, FieldNames, Debug, Clone)]
pub struct Engagement {
    pub version: String,
    pub security: Security,
    pub connection_methods: Option<ConnectionMethods>,
    pub server_retrieval_methods: Option<ServerRetrievalMethods>,
    pub protocol_info: Option<ProtocolInfo>,
}

pub type Security = CborSeq<SecurityKeyed>;

#[derive(Serialize, Deserialize, FieldNames, Debug, Clone)]
pub struct SecurityKeyed {
    pub cipher_suite_identifier: u64,
    pub e_sender_key_bytes: ESenderKeyBytes,
}

// Called DeviceRetrievalMethods in ISO 18013-5
pub type ConnectionMethods = Vec<ConnectionMethod>;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServerRetrievalMethods {
    pub web_api: WebApi,
    pub oidc: Oidc,
}

pub type Oidc = CborSeq<WebSessionInfo>;

pub type WebApi = CborSeq<WebSessionInfo>;

#[derive(Serialize, Deserialize, FieldNames, Debug, Clone)]
pub struct WebSessionInfo {
    pub version: u64,
    pub issuer_url: String,
    pub server_retrieval_token: String,
}

pub type ProtocolInfo = Value;

// Called DeviceRetrievalMethod in ISO 18013-5
pub type ConnectionMethod = CborSeq<ConnectionMethodKeyed>;

#[derive(Serialize, Deserialize, FieldNames, Debug, Clone)]
pub struct ConnectionMethodKeyed {
    #[serde(rename = "type")]
    pub typ: RequiredValue<RestApiType>,
    pub version: RequiredValue<RestApiOptionsVersion>,
    pub connection_options: CborSeq<RestApiOptionsKeyed>,
}

#[derive(Debug, Clone)]
pub struct RestApiType {}
impl RequiredValueTrait for RestApiType {
    type Type = u64;
    const REQUIRED_VALUE: Self::Type = 4;
}

#[derive(Debug, Clone)]
pub struct RestApiOptionsVersion {}
impl RequiredValueTrait for RestApiOptionsVersion {
    type Type = u64;
    const REQUIRED_VALUE: Self::Type = 1;
}

#[derive(Serialize, Deserialize, FieldNames, Debug, Clone)]
pub struct RestApiOptionsKeyed {
    uri: String,
}

pub type ESenderKeyBytes = TaggedBytes<CoseKey>;