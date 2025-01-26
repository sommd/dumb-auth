use std::sync::Arc;

use axum::http::{header, HeaderMap, HeaderValue};
use axum_extra::headers::{authorization::Basic, Authorization, HeaderMapExt};

use crate::{
    auth::{methods::AuthMethod, AuthResult},
    config::AuthConfig,
    passwords::PasswordChecker,
    AppError,
};

pub struct BasicAuth {
    password_checker: Arc<PasswordChecker>,
}

impl BasicAuth {
    pub fn new(password_checker: Arc<PasswordChecker>) -> Self {
        Self { password_checker }
    }
}

impl AuthMethod for BasicAuth {
    fn is_allowed(&self, auth_config: &AuthConfig) -> bool {
        auth_config.allow_basic
    }

    async fn verify(
        &self,
        auth_config: &AuthConfig,
        _original_uri: &str,
        headers: &HeaderMap,
    ) -> Result<AuthResult, AppError> {
        if let Some(authorization) = headers.typed_get::<Authorization<Basic>>() {
            if self
                .password_checker
                .check_password(authorization.password(), &auth_config.password)
                .await
            {
                return Ok(AuthResult::valid());
            }
        }

        Ok(AuthResult::invalid().with_header(
            header::WWW_AUTHENTICATE,
            HeaderValue::from_static("Basic realm=\"dumb-auth\""),
        ))
    }
}
