use std::env;

use assert_matches::assert_matches;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use p256::{ecdsa::SigningKey, elliptic_curve::rand_core::OsRng, pkcs8::EncodePrivateKey};
use regex::Regex;
use reqwest::header::HeaderValue;
use serial_test::serial;
use tokio::fs;

use wallet::{
    errors::ConfigurationError,
    mock::default_configuration,
    wallet_deps::{
        ConfigServerConfiguration, ConfigurationRepository, ConfigurationUpdateState, HttpConfigurationRepository,
        UpdateableConfigurationRepository,
    },
};
use wallet_common::jwt::JwtError;

use crate::common::*;

pub mod common;

#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "db_test"), ignore)]
async fn test_wallet_config() {
    let mut served_wallet_config = default_configuration();
    served_wallet_config.lock_timeouts.inactive_timeout = 1;
    served_wallet_config.lock_timeouts.background_timeout = 1;
    served_wallet_config.version = 2;

    let cs_settings = config_server_settings();
    let config_server_config = ConfigServerConfiguration {
        base_url: local_config_base_url(&cs_settings.port),
        ..Default::default()
    };
    start_config_server(cs_settings, config_jwt(&served_wallet_config)).await;

    let settings = wallet_provider_settings();
    start_wallet_provider(settings.clone()).await;

    let mut wallet_config = default_configuration();
    wallet_config.account_server.base_url = local_wp_base_url(&settings.webserver.port);

    let storage_path = env::temp_dir();
    let etag_file = storage_path.join("latest-configuration-etag.txt");
    // make sure there are no storage files from previous test runs
    let _ = fs::remove_file(etag_file.as_path()).await;

    let http_config = HttpConfigurationRepository::new(
        config_server_config.base_url,
        config_server_config.signing_public_key.into(),
        storage_path.clone(),
        wallet_config,
    )
    .await
    .unwrap();

    let before = http_config.config();
    let result = http_config.fetch().await.unwrap();
    let after = http_config.config();

    assert_matches!(result, ConfigurationUpdateState::Updated);
    assert_ne!(before.lock_timeouts, after.lock_timeouts);

    let content = fs::read(etag_file.as_path()).await.unwrap();
    let header_value = HeaderValue::from_bytes(&content).unwrap();

    let quoted_hash_regex = Regex::new(r#"".+""#).unwrap();
    assert!(quoted_hash_regex.is_match(header_value.to_str().unwrap()));

    // Second fetch should use earlier etag
    let result = http_config.fetch().await.unwrap();
    assert_matches!(result, ConfigurationUpdateState::Unmodified);
}

#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "db_test"), ignore)]
async fn test_wallet_config_stale() {
    let settings = wallet_provider_settings();

    let mut served_wallet_config = default_configuration();
    served_wallet_config.account_server.base_url = local_wp_base_url(&settings.webserver.port);

    let cs_settings = config_server_settings();
    let config_server_config = ConfigServerConfiguration {
        base_url: local_config_base_url(&cs_settings.port),
        ..Default::default()
    };
    start_config_server(cs_settings, config_jwt(&served_wallet_config)).await;

    start_wallet_provider(settings.clone()).await;

    let mut wallet_config = default_configuration();
    wallet_config.account_server.base_url = local_wp_base_url(&settings.webserver.port);

    let http_config = HttpConfigurationRepository::new(
        config_server_config.base_url,
        config_server_config.signing_public_key.into(),
        env::temp_dir(),
        wallet_config,
    )
    .await
    .unwrap();

    let before = http_config.config();
    let result = http_config.fetch().await.unwrap();
    let after = http_config.config();

    assert_matches!(result, ConfigurationUpdateState::Unmodified);
    assert_eq!(before.version, after.version);
}

#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "db_test"), ignore)]
async fn test_wallet_config_signature_verification_failed() {
    let settings = wallet_provider_settings();

    let mut served_wallet_config = default_configuration();
    served_wallet_config.account_server.base_url = local_wp_base_url(&settings.webserver.port);
    // set the wallet_config that will be return from the config server to a lower version number than
    // we already have in the default configuration
    served_wallet_config.version = 0;

    let cs_settings = config_server_settings();
    let config_server_config = ConfigServerConfiguration {
        base_url: local_config_base_url(&cs_settings.port),
        ..Default::default()
    };

    let signing_key = SigningKey::random(&mut OsRng);
    let pkcs8_der = signing_key.to_pkcs8_der().unwrap();
    let jwt = jsonwebtoken::encode(
        &Header {
            alg: Algorithm::ES256,
            ..Default::default()
        },
        &served_wallet_config,
        &EncodingKey::from_ec_der(pkcs8_der.as_bytes()),
    )
    .unwrap()
    .into_bytes();
    // Serve a wallet configuration as JWT signed by a random key
    start_config_server(cs_settings, jwt).await;

    start_wallet_provider(settings.clone()).await;

    let mut wallet_config = default_configuration();
    wallet_config.account_server.base_url = local_wp_base_url(&settings.webserver.port);

    let http_config = HttpConfigurationRepository::new(
        config_server_config.base_url,
        config_server_config.signing_public_key.into(),
        env::temp_dir(),
        wallet_config,
    )
    .await
    .unwrap();

    let result = http_config
        .fetch()
        .await
        .expect_err("Expecting invalid signature error");

    assert_matches!(result, ConfigurationError::Jwt(JwtError::Validation(e)) if *e.kind() == jsonwebtoken::errors::ErrorKind::InvalidSignature);
}
