//! Holder software, containing a [`Wallet`] that can store, receive, and disclose mdocs.
//! See [`Storage`], [`Wallet::do_issuance()`], and [`Wallet::disclose()`] respectively.

use crate::{iso::*, utils::x509::CertificateError};

pub mod disclosure;
pub use disclosure::*;

pub mod issuance;
pub use issuance::*;

pub mod mdocs;
pub use mdocs::*;

#[derive(thiserror::Error, Debug)]
pub enum HolderError {
    #[error("unsatisfiable request: DocType {0} not in wallet")]
    UnsatisfiableRequest(DocType),
    #[error("readerAuth not present for all documents")]
    ReaderAuthMissing,
    #[error("document requests were signed by different readers")]
    ReaderAuthsInconsistent,
    #[error("issuer not trusted for doctype {0}")]
    UntrustedIssuer(DocType),
    #[error("certificate error: {0}")]
    CertificateError(#[from] CertificateError),
    #[error("wrong private key type")]
    PrivateKeyTypeMismatch { expected: String, have: String },
}