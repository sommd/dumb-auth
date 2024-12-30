use std::sync::Arc;

use axum::extract::State;
use axum::http::{header, HeaderMap};
use axum::response::Response;
use axum::{http::StatusCode, response::IntoResponse};

use crate::config::Config;
use crate::sessions::Sessions;
use crate::{basic, bearer, cookie};

pub enum AuthResult {
    Missing,
    Invalid,
    Valid,
}

pub async fn handle_auth_request(
    State(config): State<Config>,
    State(sessions): State<Arc<Sessions>>,
    headers: HeaderMap,
) -> axum::response::Result<Response> {
    let original_uri = headers
        .get("X-Original-URI")
        .ok_or(StatusCode::BAD_REQUEST)?
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    if config.allow_basic {
        match basic::validate_basic_auth(&config, &headers) {
            AuthResult::Missing => {}
            AuthResult::Invalid => return Ok(StatusCode::UNAUTHORIZED.into_response()),
            AuthResult::Valid => return Ok(StatusCode::OK.into_response()),
        }
    }

    if config.allow_bearer {
        match bearer::validate_bearer_token(&config, &headers) {
            AuthResult::Missing => {}
            AuthResult::Invalid => return Ok(StatusCode::UNAUTHORIZED.into_response()),
            AuthResult::Valid => return Ok(StatusCode::OK.into_response()),
        }
    }

    match cookie::validate_session(&config, &sessions, &headers).await {
        AuthResult::Missing | AuthResult::Invalid => {
            let query = serde_urlencoded::to_string([("redirect_to", original_uri)])
                .expect("Unable to serialize query string");

            Ok((
                StatusCode::UNAUTHORIZED,
                [(header::LOCATION, format!("/login?{}", query))],
            )
                .into_response())
        }
        AuthResult::Valid => Ok(StatusCode::OK.into_response()),
    }
}
