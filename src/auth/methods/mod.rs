use axum::http::HeaderMap;

use crate::{auth::AuthResult, AppError, AuthConfig};

pub use self::basic::BasicAuth;
pub use self::bearer::BearerAuth;
pub use self::session::SessionAuth;

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
