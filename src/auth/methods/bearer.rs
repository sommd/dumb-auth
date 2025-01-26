use std::sync::Arc;

use axum::http::{header, HeaderMap, HeaderValue};
use axum_extra::headers::{authorization::Bearer, Authorization, HeaderMapExt};

use crate::{
    auth::{methods::AuthMethod, AuthResult},
    config::AuthConfig,
    passwords::PasswordChecker,
    AppError,
};

pub struct BearerAuth {
    password_checker: Arc<PasswordChecker>,
}

impl BearerAuth {
    pub fn new(password_checker: Arc<PasswordChecker>) -> Self {
        Self { password_checker }
    }
}

impl AuthMethod for BearerAuth {
    fn is_allowed(&self, auth_config: &AuthConfig) -> bool {
        auth_config.allow_bearer
    }

    async fn verify(
        &self,
        auth_config: &AuthConfig,
        _original_uri: &str,
        headers: &HeaderMap,
    ) -> Result<AuthResult, AppError> {
        if let Some(authorization) = headers.typed_get::<Authorization<Bearer>>() {
            if self
                .password_checker
                .check_password(authorization.token(), &auth_config.password)
                .await
            {
                Ok(AuthResult::valid())
            } else {
                Ok(AuthResult::invalid().with_header(
                    header::WWW_AUTHENTICATE,
                    HeaderValue::from_static("Bearer realm=\"dumb-auth\", error=\"invalid_token\""),
                ))
            }
        } else {
            Ok(AuthResult::invalid().with_header(
                header::WWW_AUTHENTICATE,
                HeaderValue::from_static("Bearer realm=\"dumb-auth\""),
            ))
        }
    }
}
