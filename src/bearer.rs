use axum::http::HeaderMap;
use axum_extra::headers::{authorization::Bearer, Authorization, HeaderMapExt};

use crate::{
    auth::AuthResult,
    password::{Password, PasswordChecker},
};

pub async fn validate_bearer_token(
    password: &Password,
    password_checker: &PasswordChecker,
    headers: &HeaderMap,
) -> AuthResult {
    if let Some(header) = headers.typed_get::<Authorization<Bearer>>() {
        if password_checker
            .check_password(header.token(), password)
            .await
        {
            AuthResult::Valid
        } else {
            AuthResult::Invalid
        }
    } else {
        AuthResult::Missing
    }
}
