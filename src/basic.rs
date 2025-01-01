use axum::http::HeaderMap;
use axum_extra::headers::{authorization::Basic, Authorization, HeaderMapExt};

use crate::{auth::AuthResult, config::AuthConfig, password};

pub fn validate_basic_auth(auth_config: &AuthConfig, headers: &HeaderMap) -> AuthResult {
    if let Some(header) = headers.typed_get::<Authorization<Basic>>() {
        if password::check_password(auth_config, header.password()) {
            AuthResult::Valid
        } else {
            AuthResult::Invalid
        }
    } else {
        AuthResult::Missing
    }
}
