use axum::http::HeaderMap;

use crate::AuthConfig;

pub use basic::BasicAuth;
pub use bearer::BearerAuth;
pub use session::SessionAuth;

use super::AuthResult;

mod basic;
mod bearer;
mod session;

pub trait AuthMethod {
    fn is_allowed(&self, auth_config: &AuthConfig) -> bool;

    async fn verify(
        &self,
        auth_config: &AuthConfig,
        original_uri: &str,
        headers: &HeaderMap,
    ) -> AuthResult;
}
