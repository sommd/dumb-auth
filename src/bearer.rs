use axum::http::HeaderMap;
use axum_extra::headers::{authorization::Bearer, Authorization, HeaderMapExt};

use crate::{auth::AuthResult, config::AuthConfig, password};

pub fn validate_bearer_token(auth_config: &AuthConfig, headers: &HeaderMap) -> AuthResult {
    if let Some(header) = headers.typed_get::<Authorization<Bearer>>() {
        if password::check_password(auth_config, header.token()) {
            AuthResult::Valid
        } else {
            AuthResult::Invalid
        }
    } else {
        AuthResult::Missing
    }
}
