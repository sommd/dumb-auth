use std::sync::Arc;

use axum::http::{header, HeaderMap, HeaderValue};
use axum_extra::headers::{Cookie, HeaderMapExt};
use tracing::{error, warn};

use crate::{
    auth::{methods::AuthMethod, AuthResult},
    config::AuthConfig,
    sessions::SessionStore,
};

pub struct SessionAuth {
    public_path: String,
    session_store: Arc<SessionStore>,
}

impl SessionAuth {
    pub fn new(public_path: String, session_store: Arc<SessionStore>) -> Self {
        Self {
            public_path,
            session_store,
        }
    }

    fn login_location(&self, original_uri: &str) -> Option<HeaderValue> {
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("redirect_to", original_uri)
            .finish();
        let redirect_uri = format!("{}/login?{}", &self.public_path, query);

        match HeaderValue::try_from(redirect_uri) {
            Ok(location) => Some(location),
            Err(e) => match HeaderValue::try_from(format!("{}/login", self.public_path)) {
                Ok(location) => {
                    warn!("Error encoding login URI with original URI: {}", e);
                    Some(location)
                }
                Err(e) => {
                    error!("Error encoding login URI: {}", e);
                    None
                }
            },
        }
    }
}

impl AuthMethod for SessionAuth {
    fn is_allowed(&self, auth_config: &AuthConfig) -> bool {
        auth_config.allow_session
    }

    async fn verify(
        &self,
        auth_config: &AuthConfig,
        original_uri: &str,
        headers: &HeaderMap,
    ) -> AuthResult {
        if let Some(cookie) = headers.typed_get::<Cookie>() {
            if let Some(session_token) = cookie.get(&auth_config.session_cookie_name) {
                if self
                    .session_store
                    .get_valid_session(session_token)
                    .await
                    .is_some()
                {
                    return AuthResult::valid();
                }
            }
        }

        if let Some(location) = self.login_location(original_uri) {
            AuthResult::invalid().with_header(header::LOCATION, location)
        } else {
            AuthResult::invalid()
        }
    }
}
