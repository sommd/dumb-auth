use std::sync::Arc;

use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};

use crate::{basic, bearer, config::AuthConfig, cookie, sessions::Sessions};

pub enum AuthResult {
    Missing,
    Invalid,
    Valid,
}

pub async fn handle_auth_request(
    State(auth_config): State<AuthConfig>,
    State(sessions): State<Arc<Sessions>>,
    headers: HeaderMap,
) -> axum::response::Result<Response> {
    let original_uri = headers
        .get("X-Original-URI")
        .ok_or(StatusCode::BAD_REQUEST)?
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    if auth_config.allow_basic {
        match basic::validate_basic_auth(&auth_config, &headers) {
            AuthResult::Missing => {}
            AuthResult::Invalid => return Ok(StatusCode::UNAUTHORIZED.into_response()),
            AuthResult::Valid => return Ok(StatusCode::OK.into_response()),
        }
    }

    if auth_config.allow_bearer {
        match bearer::validate_bearer_token(&auth_config, &headers) {
            AuthResult::Missing => {}
            AuthResult::Invalid => return Ok(StatusCode::UNAUTHORIZED.into_response()),
            AuthResult::Valid => return Ok(StatusCode::OK.into_response()),
        }
    }

    if auth_config.allow_session {
        match cookie::validate_session(&auth_config, &sessions, &headers).await {
            AuthResult::Missing | AuthResult::Invalid => {
                let query = form_urlencoded::Serializer::new(String::new())
                    .append_pair("redirect_to", original_uri)
                    .finish();

                return Ok((
                    StatusCode::UNAUTHORIZED,
                    [(header::LOCATION, format!("/login?{}", query))],
                )
                    .into_response());
            }
            AuthResult::Valid => return Ok(StatusCode::OK.into_response()),
        }
    }

    Ok(StatusCode::UNAUTHORIZED.into_response())
}
