use axum::http::HeaderMap;
use axum_extra::headers::{authorization::Bearer, Authorization, HeaderMapExt};

use crate::{auth::AuthResult, config::Config, password};

pub fn validate_bearer_token(config: &Config, headers: &HeaderMap) -> AuthResult {
    if let Some(header) = headers.typed_get::<Authorization<Bearer>>() {
        if password::check_password(config, header.token()) {
            AuthResult::Valid
        } else {
            AuthResult::Invalid
        }
    } else {
        AuthResult::Missing
    }
}
