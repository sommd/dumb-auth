use axum::http::HeaderMap;
use axum_extra::headers::{authorization::Basic, Authorization, HeaderMapExt};

use crate::{
    auth::AuthResult,
    password::{Password, PasswordChecker},
};

pub async fn validate_basic_auth(
    password: &Password,
    password_checker: &PasswordChecker,
    headers: &HeaderMap,
) -> AuthResult {
    if let Some(header) = headers.typed_get::<Authorization<Basic>>() {
        if password_checker.check_password(header.password(), password).await {
            AuthResult::Valid
        } else {
            AuthResult::Invalid
        }
    } else {
        AuthResult::Missing
    }
}
