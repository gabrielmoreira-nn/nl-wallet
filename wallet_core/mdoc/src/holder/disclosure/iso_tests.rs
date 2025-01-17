use indexmap::IndexMap;

use crate::{
    errors::Result,
    examples::{
        Example, Examples, IsoCertTimeGenerator, EXAMPLE_ATTR_NAME, EXAMPLE_ATTR_VALUE, EXAMPLE_DOC_TYPE,
        EXAMPLE_NAMESPACE,
    },
    iso::{
        device_retrieval::{DeviceRequest, ItemsRequest, ReaderAuthenticationBytes},
        disclosure::DeviceResponse,
        engagement::DeviceAuthenticationBytes,
    },
    mock::{self, DebugCollapseBts, SoftwareKeyFactory},
    SessionTranscript,
};

use super::{request::DeviceRequestMatch, test_utils::*};

/// This function uses the `MockMdocDataSource` to provide the mdoc from the example
/// `DeviceResponse` in the standard. This is used to match against a `DeviceRequest`
/// and produce a `ProposedDocument`, which in turn is converted to a `DeviceResponse`
/// by signing it.
async fn create_example_device_response(
    device_request: &DeviceRequest,
    session_transcript: SessionTranscript,
) -> Result<DeviceResponse> {
    let request_match = device_request
        .match_stored_documents(&MockMdocDataSource::default(), session_transcript)
        .await
        .unwrap();
    let proposed_document = match request_match {
        DeviceRequestMatch::Candidates(mut candidates) => candidates.remove(EXAMPLE_DOC_TYPE).unwrap().pop().unwrap(),
        _ => panic!("should have found a valid candidate in DeviceRequest"),
    };

    let device_response =
        DeviceResponse::from_proposed_documents(vec![proposed_document], &SoftwareKeyFactory::default())
            .await
            .unwrap();

    Ok(device_response)
}

/// Construct the example mdoc from the standard and disclose attributes
/// by running the example device request from the standard against it.
#[tokio::test]
async fn do_and_verify_iso_example_disclosure() {
    let device_request = DeviceRequest::example();

    // Examine some fields in the device request
    let items_request = device_request.doc_requests.first().unwrap().items_request.0.clone();
    assert_eq!(items_request.doc_type, EXAMPLE_DOC_TYPE);
    let requested_attrs = items_request.name_spaces.get(EXAMPLE_NAMESPACE).unwrap();
    let intent_to_retain = requested_attrs.get(EXAMPLE_ATTR_NAME).unwrap();
    assert!(intent_to_retain);
    println!("DeviceRequest: {:#?}", DebugCollapseBts::from(&device_request));

    // Verify reader's request
    let reader_trust_anchors = Examples::reader_trust_anchors();
    let session_transcript = ReaderAuthenticationBytes::example().0 .0.session_transcript;
    let certificate = device_request
        .doc_requests
        .first()
        .unwrap()
        .verify(session_transcript.clone(), &IsoCertTimeGenerator, reader_trust_anchors)
        .unwrap();
    let reader_x509_subject = certificate.unwrap().subject();

    // The reader's certificate contains who it is
    assert_eq!(
        reader_x509_subject.as_ref().unwrap().first().unwrap(),
        (&"CN".to_string(), &"reader".to_string())
    );
    println!("Reader: {:#?}", reader_x509_subject);

    // Construct a new `DeviceResponse`, based on the mdoc from the example device response in the standard.
    let resp = create_example_device_response(&device_request, session_transcript.clone())
        .await
        .unwrap();
    println!("DeviceResponse: {:#?}", DebugCollapseBts::from(&resp));

    // Verify this second `DeviceResponse`.
    let disclosed_attrs = resp
        .verify(
            None,
            &session_transcript,
            &IsoCertTimeGenerator,
            Examples::iaca_trust_anchors(),
        )
        .unwrap();
    println!("DisclosedAttributes: {:#?}", DebugCollapseBts::from(&disclosed_attrs));

    // The first disclosed attribute is the same as we saw earlier in the DeviceRequest
    mock::assert_disclosure_contains(
        &disclosed_attrs,
        EXAMPLE_DOC_TYPE,
        EXAMPLE_NAMESPACE,
        EXAMPLE_ATTR_NAME,
        &EXAMPLE_ATTR_VALUE,
    );
}

/// Disclose some of the attributes of the example mdoc from the spec.
#[tokio::test]
async fn iso_examples_custom_disclosure() {
    let request = DeviceRequest::new(vec![ItemsRequest {
        doc_type: EXAMPLE_DOC_TYPE.to_string(),
        name_spaces: IndexMap::from([(
            EXAMPLE_NAMESPACE.to_string(),
            IndexMap::from([(EXAMPLE_ATTR_NAME.to_string(), false)]),
        )]),
        request_info: None,
    }]);
    println!("My Request: {:#?}", DebugCollapseBts::from(&request));

    let session_transcript = DeviceAuthenticationBytes::example().0 .0.session_transcript;
    let resp = create_example_device_response(&request, session_transcript.clone())
        .await
        .unwrap();
    println!("My DeviceResponse: {:#?}", DebugCollapseBts::from(&resp));

    let disclosed_attrs = resp
        .verify(
            None,
            &session_transcript,
            &IsoCertTimeGenerator,
            Examples::iaca_trust_anchors(),
        )
        .unwrap();
    println!("My Disclosure: {:#?}", DebugCollapseBts::from(&disclosed_attrs));

    // The first disclosed attribute is the one we requested in our device request
    mock::assert_disclosure_contains(
        &disclosed_attrs,
        EXAMPLE_DOC_TYPE,
        EXAMPLE_NAMESPACE,
        EXAMPLE_ATTR_NAME,
        &EXAMPLE_ATTR_VALUE,
    );
}
