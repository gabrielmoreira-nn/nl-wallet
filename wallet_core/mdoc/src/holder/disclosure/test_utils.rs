use std::{collections::HashSet, fmt, iter, sync::Arc};

use futures::future;
use indexmap::{IndexMap, IndexSet};
use p256::{ecdsa::SigningKey, SecretKey};
use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::mpsc;
use url::Url;
use webpki::TrustAnchor;

use wallet_common::trust_anchor::DerTrustAnchor;

use crate::{
    errors::Result,
    examples::{Examples, EXAMPLE_DOC_TYPE, EXAMPLE_NAMESPACE},
    holder::{HttpClient, HttpClientError, HttpClientResult, Mdoc},
    identifiers::AttributeIdentifier,
    iso::{
        device_retrieval::{
            DeviceRequest, DeviceRequestVersion, DocRequest, ItemsRequest, ReaderAuthenticationBytes,
            ReaderAuthenticationKeyed,
        },
        disclosure::{SessionData, SessionStatus},
        engagement::{DeviceEngagement, ReaderEngagement, SessionTranscript},
    },
    mock,
    server_keys::PrivateKey,
    utils::{
        cose::{self, MdocCose},
        crypto::{SessionKey, SessionKeyUser},
        reader_auth::{reader_registration_mock, ReaderRegistration},
        serialization::{self, CborSeq, TaggedBytes},
        x509::{Certificate, CertificateType},
    },
    verifier::SessionType,
};

use super::{proposed_document::ProposedDocument, DisclosureSession, MdocDataSource, StoredMdoc};

// Constants for testing.
pub const RP_CA_CN: &str = "ca.rp.example.com";
pub const RP_CERT_CN: &str = "cert.rp.example.com";
pub const SESSION_URL: &str = "http://example.com/disclosure";
pub const RETURN_URL: &str = "http://example.com/return";

// Describe what is in `DeviceResponse::example()`.
pub const EXAMPLE_ATTRIBUTES: [&str; 5] = [
    "family_name",
    "issue_date",
    "expiry_date",
    "document_number",
    "driving_privileges",
];

pub type MdocIdentifier = String;

/// Build an [`ItemsRequest`] from a list of attributes.
pub fn items_request(
    doc_type: String,
    name_space: String,
    attributes: impl Iterator<Item = impl Into<String>>,
) -> ItemsRequest {
    ItemsRequest {
        doc_type,
        name_spaces: IndexMap::from_iter([(
            name_space,
            attributes.map(|attribute| (attribute.into(), false)).collect(),
        )]),
        request_info: None,
    }
}

pub fn example_items_request() -> ItemsRequest {
    items_request(
        EXAMPLE_DOC_TYPE.to_string(),
        EXAMPLE_NAMESPACE.to_string(),
        EXAMPLE_ATTRIBUTES.iter().copied(),
    )
}

pub fn emtpy_items_request() -> ItemsRequest {
    items_request(
        EXAMPLE_DOC_TYPE.to_string(),
        EXAMPLE_NAMESPACE.to_string(),
        iter::empty::<String>(),
    )
}

/// Convenience function for creating a [`PrivateKey`],
/// based on a CA certificate and signing key.
pub fn create_private_key(
    ca: &Certificate,
    ca_signing_key: &SigningKey,
    reader_registration: Option<ReaderRegistration>,
) -> PrivateKey {
    let (certificate, signing_key) = Certificate::new(
        ca,
        ca_signing_key,
        RP_CERT_CN,
        CertificateType::ReaderAuth(reader_registration.map(Box::new)),
    )
    .unwrap();

    PrivateKey::new(signing_key, certificate)
}

/// Create a basic `SessionTranscript` we can use for testing.
pub fn create_basic_session_transcript() -> SessionTranscript {
    let (reader_engagement, _reader_private_key) =
        ReaderEngagement::new_reader_engagement(SESSION_URL.parse().unwrap()).unwrap();
    let (device_engagement, _device_private_key) =
        DeviceEngagement::new_device_engagement("https://example.com".parse().unwrap()).unwrap();

    SessionTranscript::new(SessionType::SameDevice, &reader_engagement, &device_engagement).unwrap()
}

/// Create a `DocRequest` including reader authentication,
/// based on a `SessionTranscript` and `PrivateKey`.
pub async fn create_doc_request(
    items_request: ItemsRequest,
    session_transcript: SessionTranscript,
    private_key: &PrivateKey,
) -> DocRequest {
    // Generate the reader authentication signature, without payload.
    let reader_auth = ReaderAuthenticationKeyed {
        reader_auth_string: Default::default(),
        session_transcript,
        items_request_bytes: items_request.clone().into(),
    };

    let cose = MdocCose::<_, ReaderAuthenticationBytes>::sign(
        &TaggedBytes(CborSeq(reader_auth)),
        cose::new_certificate_header(&private_key.cert_bts),
        private_key,
        false,
    )
    .await
    .unwrap();

    // Create and encrypt the `DeviceRequest`.
    DocRequest {
        items_request: items_request.into(),
        reader_auth: Some(cose.0.into()),
    }
}

/// Create the example `Mdoc`.
pub fn create_example_mdoc() -> Mdoc {
    let trust_anchors = Examples::iaca_trust_anchors();
    mock::mdoc_from_example_device_response(trust_anchors)
}

/// Create `ProposedDocument` based on the example `Mdoc`.
pub fn create_example_proposed_document() -> ProposedDocument<MdocIdentifier> {
    let mdoc = create_example_mdoc();

    ProposedDocument {
        source_identifier: "id_1234".to_string(),
        private_key_id: mdoc.private_key_id,
        doc_type: mdoc.doc_type,
        issuer_signed: mdoc.issuer_signed,
        device_signed_challenge: b"signing_challenge".to_vec(),
    }
}

/// The `AttributeIdentifier`s contained in the example `Mdoc`.
pub fn example_mdoc_attribute_identifiers() -> IndexSet<AttributeIdentifier> {
    create_example_mdoc().issuer_signed_attribute_identifiers()
}

/// Create an ordered set of `AttributeIdentifier`s within the
/// example doc type and name space for a given set of attributes.
pub fn example_identifiers_from_attributes(
    attributes: impl IntoIterator<Item = impl Into<String>>,
) -> IndexSet<AttributeIdentifier> {
    attributes
        .into_iter()
        .map(|attribute| AttributeIdentifier {
            doc_type: EXAMPLE_DOC_TYPE.to_string(),
            namespace: EXAMPLE_NAMESPACE.to_string(),
            attribute: attribute.into(),
        })
        .collect()
}

/// An implementor of `HttpClient` that either returns `SessionData`
/// with a particular `SessionStatus` or returns an error. Messages sent
/// through this `HttpClient` can be inspected through a `mpsc` channel.
pub struct MockHttpClient<F> {
    pub response_factory: F,
    pub payload_sender: mpsc::Sender<Vec<u8>>,
}

pub enum MockHttpClientResponse {
    Error(HttpClientError),
    SessionStatus(SessionStatus),
}

impl<F> fmt::Debug for MockHttpClient<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MockHttpClient")
            .field("payload_sender", &self.payload_sender)
            .finish_non_exhaustive()
    }
}

impl<F> HttpClient for MockHttpClient<F>
where
    F: Fn() -> MockHttpClientResponse,
{
    async fn post<R, V>(&self, _url: &Url, val: &V) -> HttpClientResult<R>
    where
        V: Serialize,
        R: DeserializeOwned,
    {
        // Serialize the payload and give it to the sender.
        _ = self
            .payload_sender
            .send(serialization::cbor_serialize(val).unwrap())
            .await;

        let response = match (self.response_factory)() {
            MockHttpClientResponse::Error(error) => return Err(error),
            MockHttpClientResponse::SessionStatus(session_status) => {
                let session_data = SessionData {
                    data: None,
                    status: session_status.into(),
                };
                serialization::cbor_deserialize(serialization::cbor_serialize(&session_data).unwrap().as_slice())
                    .unwrap()
            }
        };

        Ok(response)
    }
}

/// A type that implements `MdocDataSource` and simply returns
/// the [`Mdoc`] contained in `DeviceResponse::example()`, if its
/// `doc_type` is requested.
#[derive(Debug)]
pub struct MockMdocDataSource {
    pub mdocs: Vec<Mdoc>,
    pub has_error: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum MdocDataSourceError {
    #[error("failed")]
    Failed,
}

impl Default for MockMdocDataSource {
    fn default() -> Self {
        MockMdocDataSource {
            mdocs: vec![create_example_mdoc()],
            has_error: false,
        }
    }
}

impl MdocDataSource for MockMdocDataSource {
    type MdocIdentifier = MdocIdentifier;
    type Error = MdocDataSourceError;

    async fn mdoc_by_doc_types(
        &self,
        doc_types: &HashSet<&str>,
    ) -> std::result::Result<Vec<Vec<StoredMdoc<Self::MdocIdentifier>>>, Self::Error> {
        if self.has_error {
            return Err(MdocDataSourceError::Failed);
        }

        if doc_types.contains(EXAMPLE_DOC_TYPE) {
            let stored_mdocs = self
                .mdocs
                .iter()
                .cloned()
                .enumerate()
                .map(|(index, mdoc)| StoredMdoc {
                    id: format!("id_{}", index + 1),
                    mdoc,
                })
                .collect();

            return Ok(vec![stored_mdocs]);
        }

        Ok(Default::default())
    }
}

/// This type contains the minimum logic to respond with the correct
/// verifier messages in a disclosure session. Currently it only responds
/// with a [`SessionData`] containing a [`DeviceRequest`].
pub struct MockVerifierSession<F> {
    pub session_type: SessionType,
    pub return_url: Option<Url>,
    pub reader_registration: Option<ReaderRegistration>,
    pub trust_anchors: Vec<DerTrustAnchor>,
    private_key: PrivateKey,
    pub reader_engagement: ReaderEngagement,
    reader_ephemeral_key: SecretKey,
    pub reader_engagement_bytes_override: Option<Vec<u8>>,
    pub items_requests: Vec<ItemsRequest>,
    transform_device_request: F,
}

impl<F> fmt::Debug for MockVerifierSession<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MockVerifierSession")
            .field("session_type", &self.session_type)
            .field("return_url", &self.return_url)
            .field("reader_registration", &self.reader_registration)
            .field("trust_anchors", &self.trust_anchors)
            .field("reader_engagement", &self.reader_engagement)
            .field(
                "reader_engagement_bytes_override",
                &self.reader_engagement_bytes_override,
            )
            .field("items_requests", &self.items_requests)
            .finish_non_exhaustive()
    }
}

impl<F> MockVerifierSession<F>
where
    F: Fn(DeviceRequest) -> DeviceRequest,
{
    pub fn new(
        session_type: SessionType,
        session_url: Url,
        return_url: Option<Url>,
        reader_registration: Option<ReaderRegistration>,
        transform_device_request: F,
    ) -> Self {
        // Generate trust anchors, signing key and certificate containing `ReaderRegistration`.
        let (ca, ca_privkey) = Certificate::new_ca(RP_CA_CN).unwrap();
        let trust_anchors = vec![DerTrustAnchor::from_der(ca.as_bytes().to_vec()).unwrap()];
        let private_key = create_private_key(&ca, &ca_privkey, reader_registration.as_ref().cloned());

        // Generate the `ReaderEngagement` that would be be sent in the UL.
        let (reader_engagement, reader_ephemeral_key) = ReaderEngagement::new_reader_engagement(session_url).unwrap();

        // Set up the default item requests
        let items_requests = vec![example_items_request()];

        MockVerifierSession {
            session_type,
            return_url,
            reader_registration,
            trust_anchors,
            private_key,
            reader_engagement,
            reader_engagement_bytes_override: None,
            reader_ephemeral_key,
            items_requests,
            transform_device_request,
        }
    }

    fn reader_engagement_bytes(&self) -> Vec<u8> {
        self.reader_engagement_bytes_override
            .as_ref()
            .cloned()
            .unwrap_or(serialization::cbor_serialize(&self.reader_engagement).unwrap())
    }

    fn trust_anchors(&self) -> Vec<TrustAnchor> {
        self.trust_anchors
            .iter()
            .map(|anchor| (&anchor.owned_trust_anchor).into())
            .collect()
    }

    // Generate the `SessionData` response containing the `DeviceRequest`,
    // based on the `DeviceEngagement` received from the device.
    async fn device_request_session_data(&self, device_engagement: DeviceEngagement) -> SessionData {
        // Create the session transcript and encryption key.
        let session_transcript =
            SessionTranscript::new(self.session_type, &self.reader_engagement, &device_engagement).unwrap();

        let device_public_key = device_engagement.0.security.as_ref().unwrap().try_into().unwrap();

        let reader_key = SessionKey::new(
            &self.reader_ephemeral_key,
            &device_public_key,
            &session_transcript,
            SessionKeyUser::Reader,
        )
        .unwrap();

        // Create a `DocRequest` for every `ItemRequest`.
        let doc_requests = future::join_all(self.items_requests.iter().cloned().map(|items_request| async {
            create_doc_request(items_request, session_transcript.clone(), &self.private_key).await
        }))
        .await;

        let device_request = (self.transform_device_request)(DeviceRequest {
            version: DeviceRequestVersion::V1_0,
            doc_requests,
        });

        SessionData::serialize_and_encrypt(&device_request, &reader_key).unwrap()
    }
}

/// This type implements [`HttpClient`] and simply forwards the
/// requests to an instance of [`MockVerifierSession`].
pub struct MockVerifierSessionClient<F> {
    session: Arc<MockVerifierSession<F>>,
    payload_sender: mpsc::Sender<Vec<u8>>,
}

impl<F> fmt::Debug for MockVerifierSessionClient<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MockVerifierSessionClient")
            .field("session", &self.session)
            .finish_non_exhaustive()
    }
}

impl<F> HttpClient for MockVerifierSessionClient<F>
where
    F: Fn(DeviceRequest) -> DeviceRequest,
{
    async fn post<R, V>(&self, url: &Url, val: &V) -> HttpClientResult<R>
    where
        V: Serialize,
        R: DeserializeOwned,
    {
        // The URL has to match the one on the configured `ReaderEngagement`.
        assert_eq!(url, self.session.reader_engagement.verifier_url().unwrap());

        // Serialize the payload and give a copy of it to the sender.
        let payload = serialization::cbor_serialize(val).unwrap();
        _ = self.payload_sender.send(payload.clone()).await;

        // Serialize and deserialize both the request and response
        // in order to adhere to the trait bounds. If the request deserializes
        // as a `DeviceEngagement` process it, otherwise terminate the session.
        let session_data = match serialization::cbor_deserialize(payload.as_slice()) {
            Ok(device_engagement) => self.session.device_request_session_data(device_engagement).await,
            Err(_) => SessionData::new_termination(),
        };

        let result =
            serialization::cbor_deserialize(serialization::cbor_serialize(&session_data).unwrap().as_slice()).unwrap();

        Ok(result)
    }
}
pub enum ReaderCertificateKind {
    NoReaderRegistration,
    WithReaderRegistration,
}

/// Perform a [`DisclosureSession`] start with test defaults.
/// This function takes several closures for modifying these
/// defaults just before they are actually used.
pub async fn disclosure_session_start<FS, FM, FD>(
    session_type: SessionType,
    certificate_kind: ReaderCertificateKind,
    payloads: &mut Vec<Vec<u8>>,
    transform_verfier_session: FS,
    transform_mdoc: FM,
    transform_device_request: FD,
) -> Result<(
    DisclosureSession<MockVerifierSessionClient<FD>, MdocIdentifier>,
    Arc<MockVerifierSession<FD>>,
    mpsc::Receiver<Vec<u8>>,
)>
where
    FS: FnOnce(MockVerifierSession<FD>) -> MockVerifierSession<FD>,
    FM: FnOnce(MockMdocDataSource) -> MockMdocDataSource,
    FD: Fn(DeviceRequest) -> DeviceRequest,
{
    // Create a reader registration with all of the example attributes,
    // if we should have a reader registration at all.
    let reader_registration = match certificate_kind {
        ReaderCertificateKind::NoReaderRegistration => None,
        ReaderCertificateKind::WithReaderRegistration => ReaderRegistration {
            attributes: mock::reader_registration_attributes(
                EXAMPLE_DOC_TYPE.to_string(),
                EXAMPLE_NAMESPACE.to_string(),
                EXAMPLE_ATTRIBUTES.iter().copied(),
            ),
            ..reader_registration_mock()
        }
        .into(),
    };

    // Create a mock session and call the transform callback.
    let verifier_session = MockVerifierSession::<FD>::new(
        SessionType::SameDevice,
        SESSION_URL.parse().unwrap(),
        Url::parse(RETURN_URL).unwrap().into(),
        reader_registration,
        transform_device_request,
    );
    let verifier_session = Arc::new(transform_verfier_session(verifier_session));

    // Create the payload channel and a mock HTTP client.
    let (payload_sender, mut payload_receiver) = mpsc::channel(256);
    let client = MockVerifierSessionClient {
        session: Arc::clone(&verifier_session),
        payload_sender,
    };

    // Set up the mock data source.
    let mdoc_data_source = transform_mdoc(MockMdocDataSource::default());

    // Starting disclosure and return the result.
    let result = DisclosureSession::start(
        client,
        &verifier_session.reader_engagement_bytes(),
        verifier_session.return_url.clone(),
        session_type,
        &mdoc_data_source,
        &verifier_session.trust_anchors(),
    )
    .await;

    while let Ok(payload) = payload_receiver.try_recv() {
        payloads.push(payload);
    }

    result.map(|disclosure_session| (disclosure_session, verifier_session, payload_receiver))
}
