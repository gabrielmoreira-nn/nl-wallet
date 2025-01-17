use std::{
    error::Error,
    net::{SocketAddr, TcpListener},
};

use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use etag::EntityTag;
use http::{header, HeaderMap, HeaderValue, StatusCode};
use tracing::{debug, info};

use super::settings::Settings;

pub async fn serve(settings: Settings, config_jwt: Vec<u8>) -> Result<(), Box<dyn Error>> {
    let socket = SocketAddr::new(settings.ip, settings.port);
    let listener = TcpListener::bind(socket)?;
    debug!("listening on {}", socket);

    let app = Router::new().nest("/", health_router()).nest(
        "/config/v1",
        Router::new()
            .route("/wallet-config", get(configuration))
            .with_state(config_jwt),
    );

    axum::Server::from_tcp(listener)?.serve(app.into_make_service()).await?;

    Ok(())
}

fn health_router() -> Router {
    Router::new().route("/health", get(|| async {}))
}

async fn configuration(
    State(config_jwt): State<Vec<u8>>,
    headers: HeaderMap,
) -> std::result::Result<Response, StatusCode> {
    info!("Received configuration request");

    let config_entity_tag = EntityTag::from_data(config_jwt.as_ref());

    if let Some(etag) = headers.get(header::IF_NONE_MATCH) {
        let entity_tag = etag
            .to_str()
            .ok()
            .and_then(|etag| etag.parse().ok())
            .ok_or(StatusCode::BAD_REQUEST)?;

        // Comparing etags using the If-None-Match header uses the weak comparison algorithm.
        if config_entity_tag.weak_eq(&entity_tag) {
            debug!("Configuration is not modified");
            return Err(StatusCode::NOT_MODIFIED);
        }
    }

    let mut resp: Response = config_jwt.into_response();
    resp.headers_mut().append(
        header::ETAG,
        // We can safely unwrap here because we know for sure there are no non-ascii characters used.
        HeaderValue::from_str(&config_entity_tag.to_string()).unwrap(),
    );

    info!("Replying with the configuration");
    Ok(resp)
}
