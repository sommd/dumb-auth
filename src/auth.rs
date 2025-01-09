use std::sync::Arc;

use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};

use crate::{basic, bearer, cookie, password::PasswordChecker, sessions::Sessions, AppConfig};

pub enum AuthResult {
    Missing,
    Invalid,
    Valid,
}

pub async fn handle_auth_request(
    State(config): State<AppConfig>,
    State(password_checker): State<Arc<PasswordChecker>>,
    State(sessions): State<Arc<Sessions>>,
    headers: HeaderMap,
) -> axum::response::Result<Response> {
    let original_uri = headers
        .get("X-Original-URI")
        .ok_or(StatusCode::BAD_REQUEST)?
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    if config.auth_config.allow_basic {
        match basic::validate_basic_auth(&config.auth_config.password, &password_checker, &headers)
            .await
        {
            AuthResult::Missing => {}
            AuthResult::Invalid => return Ok(StatusCode::UNAUTHORIZED.into_response()),
            AuthResult::Valid => return Ok(StatusCode::OK.into_response()),
        }
    }

    if config.auth_config.allow_bearer {
        match bearer::validate_bearer_token(
            &config.auth_config.password,
            &password_checker,
            &headers,
        )
        .await
        {
            AuthResult::Missing => {}
            AuthResult::Invalid => return Ok(StatusCode::UNAUTHORIZED.into_response()),
            AuthResult::Valid => return Ok(StatusCode::OK.into_response()),
        }
    }

    if config.auth_config.allow_session {
        match cookie::validate_session(&config.auth_config, &sessions, &headers).await {
            AuthResult::Missing | AuthResult::Invalid => {
                let query = form_urlencoded::Serializer::new(String::new())
                    .append_pair("redirect_to", original_uri)
                    .finish();

                return Ok((
                    StatusCode::UNAUTHORIZED,
                    [(
                        header::LOCATION,
                        format!("{}/login?{}", config.public_path, query),
                    )],
                )
                    .into_response());
            }
            AuthResult::Valid => return Ok(StatusCode::OK.into_response()),
        }
    }

    Ok(StatusCode::UNAUTHORIZED.into_response())
}
