use axum::http::{header::IntoHeaderName, HeaderMap, HeaderValue};

pub(crate) use self::{auth_request::handle_auth_request, authenticator::Authenticator};

mod auth_request;
mod authenticator;
mod methods;

pub struct AuthResult {
    pub valid: bool,
    pub response_headers: Option<HeaderMap>,
}

impl AuthResult {
    pub fn valid() -> Self {
        Self {
            valid: true,
            response_headers: None,
        }
    }

    pub fn invalid() -> Self {
        Self {
            valid: false,
            response_headers: None,
        }
    }

    pub fn with_header(mut self, key: impl IntoHeaderName, value: HeaderValue) -> Self {
        self.response_headers
            .get_or_insert_with(|| HeaderMap::with_capacity(1))
            .append(key, value);
        self
    }
}
