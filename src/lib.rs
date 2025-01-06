use std::sync::Arc;

use axum::{
    extract::FromRef,
    routing::{any, get},
    Router,
};
use password::PasswordChecker;

use crate::sessions::Sessions;

pub use crate::config::*;
pub use crate::login::LoginForm;
pub use crate::password::{hash_password, Password};

mod auth;
mod basic;
mod bearer;
mod config;
mod cookie;
mod login;
mod password;
mod sessions;

#[derive(Clone)]
pub(crate) struct AppState {
    auth_config: AuthConfig,
    password_checker: Arc<PasswordChecker>,
    sessions: Arc<Sessions>,
}

impl FromRef<AppState> for AuthConfig {
    fn from_ref(input: &AppState) -> Self {
        input.auth_config.clone()
    }
}

impl FromRef<AppState> for Arc<PasswordChecker> {
    fn from_ref(input: &AppState) -> Self {
        input.password_checker.clone()
    }
}

impl FromRef<AppState> for Arc<Sessions> {
    fn from_ref(input: &AppState) -> Self {
        input.sessions.clone()
    }
}

pub fn app(auth_config: AuthConfig) -> Router {
    let password_checker = Arc::new(PasswordChecker::default());
    let sessions = Arc::new(Sessions::new(auth_config.session_expiry));

    Router::new()
        .route("/auth", any(auth::handle_auth_request))
        .route(
            "/login",
            get(login::handle_get_login).post(login::handle_post_login),
        )
        .with_state(AppState {
            auth_config,
            password_checker,
            sessions,
        })
}
