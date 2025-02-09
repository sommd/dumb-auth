use std::sync::Arc;

use axum::http::HeaderMap;
use tracing::{debug, instrument};

use crate::{passwords::PasswordChecker, sessions::SessionManager, AppError, AuthConfig};

use super::{
    methods::{AuthMethod, BasicAuth, BearerAuth, SessionAuth},
    AuthResult,
};

pub(crate) struct Authenticator {
    basic: BasicAuth,
    bearer: BearerAuth,
    session: SessionAuth,
}

impl Authenticator {
    pub fn new(
        public_path: String,
        password_checker: Arc<PasswordChecker>,
        session_manager: Arc<SessionManager>,
    ) -> Self {
        Self {
            basic: BasicAuth::new(password_checker.clone()),
            bearer: BearerAuth::new(password_checker),
            session: SessionAuth::new(public_path, session_manager),
        }
    }

    #[instrument(skip(self, auth_config, headers))]
    pub async fn authenticate(
        &self,
        auth_config: &AuthConfig,
        original_uri: &str,
        headers: &HeaderMap,
    ) -> Result<AuthResult, AppError> {
        let result = self
            .do_authenticate(auth_config, original_uri, headers)
            .await?;

        debug!("Auth: {}", if result.valid { "valid" } else { "invalid" });

        Ok(result)
    }

    async fn do_authenticate(
        &self,
        auth_config: &AuthConfig,
        original_uri: &str,
        headers: &HeaderMap,
    ) -> Result<AuthResult, AppError> {
        let mut all_response_headers = None;

        if self.basic.is_allowed(auth_config) {
            match self
                .basic
                .verify(auth_config, original_uri, headers)
                .await?
            {
                result @ AuthResult { valid: true, .. } => return Ok(result),
                result => Self::append_result(&mut all_response_headers, result),
            }
        }

        if self.bearer.is_allowed(auth_config) {
            match self
                .bearer
                .verify(auth_config, original_uri, headers)
                .await?
            {
                result @ AuthResult { valid: true, .. } => return Ok(result),
                result => Self::append_result(&mut all_response_headers, result),
            }
        }

        if self.session.is_allowed(auth_config) {
            match self
                .session
                .verify(auth_config, original_uri, headers)
                .await?
            {
                result @ AuthResult { valid: true, .. } => return Ok(result),
                result => Self::append_result(&mut all_response_headers, result),
            }
        }

        Ok(AuthResult {
            valid: false,
            response_headers: all_response_headers,
        })
    }

    fn append_result(all_response_headers: &mut Option<HeaderMap>, auth_result: AuthResult) {
        if let Some(response_headers) = auth_result.response_headers {
            if let Some(all_response_headers) = all_response_headers {
                all_response_headers.extend(response_headers);
            } else {
                all_response_headers.replace(response_headers);
            }
        }
    }
}
