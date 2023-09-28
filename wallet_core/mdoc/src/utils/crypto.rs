//! Cryptographic utilities: SHA256, ECDSA, Diffie-Hellman, HKDF, and key conversion functions.

use aes_gcm::{
    aead::{Aead, Nonce},
    Aes256Gcm, Key, KeyInit,
};
use ciborium::value::Value;
use coset::{iana, CoseKeyBuilder, Label};
use p256::{
    ecdh::{self, EphemeralSecret},
    ecdsa::{SigningKey, VerifyingKey},
    EncodedPoint, PublicKey,
};
use ring::hmac;
use serde::Serialize;
use serde_bytes::ByteBuf;
use x509_parser::nom::AsBytes;

use wallet_common::utils::{hkdf, sha256};

use crate::{
    utils::{
        cose::CoseKey,
        serialization::{cbor_serialize, CborError, TaggedBytes},
    },
    Error, Result, SessionData, SessionTranscript,
};

#[derive(thiserror::Error, Debug)]
pub enum CryptoError {
    #[error("HKDF failed")]
    Hkdf,
    #[error("missing coordinate")]
    KeyMissingCoordinate,
    #[error("wrong key type")]
    KeyWrongType,
    #[error("missing key ID")]
    KeyMissingKeyID,
    #[error("unexpected COSE_Key label")]
    KeyUnepectedCoseLabel,
    #[error("coordinate parse failed")]
    KeyCoordinateParseFailed,
    #[error("key parse failed: {0}")]
    KeyParseFailed(#[from] p256::ecdsa::Error),
    #[error("AES encryption/decryption failed")]
    Aes,
}

/// Computes the SHA256 of the CBOR encoding of the argument.
pub fn cbor_digest<T: Serialize>(val: &T) -> std::result::Result<Vec<u8>, CborError> {
    let digest = sha256(cbor_serialize(val)?.as_ref());
    Ok(digest)
}

/// Using Diffie-Hellman and the HKDF from RFC 5869, compute a HMAC key.
pub fn dh_hmac_key(
    privkey: &SigningKey,
    pubkey: &VerifyingKey,
    salt: &[u8],
    info: &str,
    len: usize,
) -> Result<hmac::Key> {
    let dh = ecdh::diffie_hellman(privkey.as_nonzero_scalar(), pubkey.as_affine());
    hmac_key(dh.raw_secret_bytes().as_ref(), salt, info, len)
}

// TODO support no salt
/// Using the HKDF from RFC 5869, compute a HMAC key.
pub fn hmac_key(input_key_material: &[u8], salt: &[u8], info: &str, len: usize) -> Result<hmac::Key> {
    let bts = hkdf(input_key_material, sha256(salt).as_slice(), info, len).map_err(|_| CryptoError::Hkdf)?;
    let key = hmac::Key::new(hmac::HMAC_SHA256, &bts);
    Ok(key)
}

impl TryFrom<&VerifyingKey> for CoseKey {
    type Error = Error;
    fn try_from(key: &VerifyingKey) -> std::result::Result<Self, Self::Error> {
        let encoded_point = key.to_encoded_point(false);
        let x = encoded_point.x().ok_or(CryptoError::KeyMissingCoordinate)?.to_vec();
        let y = encoded_point.y().ok_or(CryptoError::KeyMissingCoordinate)?.to_vec();

        let key = CoseKey(CoseKeyBuilder::new_ec2_pub_key(iana::EllipticCurve::P_256, x, y).build());
        Ok(key)
    }
}

impl TryFrom<&CoseKey> for VerifyingKey {
    type Error = Error;
    fn try_from(key: &CoseKey) -> Result<Self> {
        if key.0.kty != coset::RegisteredLabel::Assigned(iana::KeyType::EC2) {
            return Err(CryptoError::KeyWrongType.into());
        }

        let keyid = key.0.params.get(0).ok_or(CryptoError::KeyMissingKeyID)?;
        if *keyid != (Label::Int(-1), Value::Integer(1.into())) {
            return Err(CryptoError::KeyWrongType.into());
        }

        let x = key.0.params.get(1).ok_or(CryptoError::KeyMissingCoordinate)?;
        if x.0 != Label::Int(-2) {
            return Err(CryptoError::KeyUnepectedCoseLabel.into());
        }
        let y = key.0.params.get(2).ok_or(CryptoError::KeyMissingCoordinate)?;
        if y.0 != Label::Int(-3) {
            return Err(CryptoError::KeyUnepectedCoseLabel.into());
        }

        let key = VerifyingKey::from_encoded_point(&EncodedPoint::from_affine_coordinates(
            x.1.as_bytes()
                .ok_or(CryptoError::KeyCoordinateParseFailed)?
                .as_bytes()
                .into(),
            y.1.as_bytes()
                .ok_or(CryptoError::KeyCoordinateParseFailed)?
                .as_bytes()
                .into(),
            false,
        ))
        .map_err(CryptoError::KeyParseFailed)?;
        Ok(key)
    }
}

pub struct SessionKey {
    key: Vec<u8>,
    user: SessionKeyUser,
}

/// Identifies which agent uses the [`SessionKey`] to encrypt its messages.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SessionKeyUser {
    Reader,
    Device,
}

impl SessionKey {
    pub fn new(
        privkey: &EphemeralSecret,
        pubkey: &PublicKey,
        session_transcript: &SessionTranscript,
        user: SessionKeyUser,
    ) -> Result<Self> {
        let dh = privkey.diffie_hellman(pubkey);
        let salt = sha256(&cbor_serialize(&TaggedBytes(session_transcript))?);
        let user_str = match user {
            SessionKeyUser::Reader => "SKReader",
            SessionKeyUser::Device => "SKDevice",
        };
        let key = hkdf(dh.raw_secret_bytes(), &salt, user_str, 32).map_err(|_| CryptoError::Hkdf)?;
        let key = SessionKey { key, user };
        Ok(key)
    }
}

impl SessionData {
    fn nonce(user: SessionKeyUser) -> Nonce<Aes256Gcm> {
        let mut nonce = vec![0u8; 12];

        if user == SessionKeyUser::Device {
            nonce[7] = 1; // the 8th byte indicates the user (0 = reader, 1 = device)
        }

        // The final byte is the message count, starting at one.
        // We will support sending a maximum of 1 message per sender.
        nonce[11] = 1;

        *Nonce::<Aes256Gcm>::from_slice(&nonce)
    }

    pub fn encrypt(data: &[u8], key: &SessionKey) -> Result<Self> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key.key.as_bytes()));
        let ciphertext = cipher
            .encrypt(&Self::nonce(key.user), data)
            .map_err(|_| CryptoError::Aes)?;

        Ok(SessionData {
            data: Some(ByteBuf::from(ciphertext)),
            status: None,
        })
    }

    pub fn decrypt(&self, key: &SessionKey) -> Result<Vec<u8>> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key.key.as_bytes()));
        let plaintext = cipher
            .decrypt(&Self::nonce(key.user), self.data.as_ref().unwrap().as_bytes())
            .map_err(|_| CryptoError::Aes)?;
        Ok(plaintext)
    }
}

#[cfg(test)]
mod test {
    use aes_gcm::aead::OsRng;
    use p256::ecdh::EphemeralSecret;

    use crate::{examples::Example, DeviceAuthenticationBytes, SessionData};

    use super::{SessionKey, SessionKeyUser};

    #[test]
    fn session_data_encryption() {
        let plaintext = b"Hello, world!";

        let device_privkey = EphemeralSecret::random(&mut OsRng);
        let reader_privkey = EphemeralSecret::random(&mut OsRng);

        let key = SessionKey::new(
            &device_privkey,
            &reader_privkey.public_key(),
            &DeviceAuthenticationBytes::example().0 .0.session_transcript,
            SessionKeyUser::Device,
        )
        .unwrap();

        let session_data = SessionData::encrypt(plaintext, &key).unwrap();
        assert!(session_data.data.is_some());
        assert!(session_data.status.is_none());

        let decrypted = session_data.decrypt(&key).unwrap();
        assert_eq!(&plaintext[..], &decrypted);
    }
}
