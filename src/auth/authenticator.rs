use std::sync::Arc;

use axum::http::HeaderMap;

use crate::{passwords::PasswordChecker, sessions::SessionStore, AuthConfig};

use super::{
    methods::{AuthMethod, BasicAuth, BearerAuth, SessionAuth},
    AuthResult,
};

pub struct Authenticator {
    basic: BasicAuth,
    bearer: BearerAuth,
    session: SessionAuth,
}

impl Authenticator {
    pub fn new(
        public_path: String,
        password_checker: Arc<PasswordChecker>,
        session_store: Arc<SessionStore>,
    ) -> Self {
        Self {
            basic: BasicAuth::new(password_checker.clone()),
            bearer: BearerAuth::new(password_checker),
            session: SessionAuth::new(public_path, session_store),
        }
    }

    pub async fn authenticate(
        &self,
        original_uri: &str,
        auth_config: &AuthConfig,
        headers: &HeaderMap,
    ) -> AuthResult {
        let mut all_response_headers = None;

        if self.basic.is_allowed(auth_config) {
            match self.basic.verify(original_uri, auth_config, headers).await {
                result @ AuthResult { valid: true, .. } => return result,
                result => Self::append_result(&mut all_response_headers, result),
            }
        }

        if self.bearer.is_allowed(auth_config) {
            match self.bearer.verify(original_uri, auth_config, headers).await {
                result @ AuthResult { valid: true, .. } => return result,
                result => Self::append_result(&mut all_response_headers, result),
            }
        }

        if self.session.is_allowed(auth_config) {
            match self
                .session
                .verify(original_uri, auth_config, headers)
                .await
            {
                result @ AuthResult { valid: true, .. } => return result,
                result => Self::append_result(&mut all_response_headers, result),
            }
        }

        AuthResult {
            valid: false,
            response_headers: all_response_headers,
        }
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
