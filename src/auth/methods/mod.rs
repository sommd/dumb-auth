use axum::http::HeaderMap;

use crate::{auth::AuthResult, AppError, AuthConfig};

pub use basic::BasicAuth;
pub use bearer::BearerAuth;
pub use session::SessionAuth;

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
    ) -> Result<AuthResult, AppError>;
}
