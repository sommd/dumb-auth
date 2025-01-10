use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use log::error;

use crate::{auth::Authenticator, AuthConfig};

const ORIGINAL_URI_HEADER: &str = "X-Original-URI";

pub async fn handle_auth_request(
    State(auth_config): State<AuthConfig>,
    State(authenticator): State<Arc<Authenticator>>,
    headers: HeaderMap,
) -> axum::response::Result<impl IntoResponse> {
    let original_uri = headers
        .get(ORIGINAL_URI_HEADER)
        .ok_or_else(|| {
            error!("Request missing {} header", ORIGINAL_URI_HEADER);
            StatusCode::BAD_REQUEST
        })?
        .to_str()
        .map_err(|e| {
            error!("Error decoding {} header: {}", ORIGINAL_URI_HEADER, e);
            StatusCode::BAD_REQUEST
        })?;

    let result = authenticator
        .authenticate(original_uri, &auth_config, &headers)
        .await;

    let status = if result.valid {
        StatusCode::OK
    } else {
        StatusCode::UNAUTHORIZED
    };

    Ok((
        status,
        result
            .response_headers
            .map_or_else(|| ().into_response(), IntoResponse::into_response),
    ))
}
