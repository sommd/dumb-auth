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
    config: AppConfig,
    password_checker: Arc<PasswordChecker>,
    sessions: Arc<Sessions>,
}

impl FromRef<AppState> for AppConfig {
    fn from_ref(input: &AppState) -> Self {
        input.config.clone()
    }
}

impl FromRef<AppState> for AuthConfig {
    fn from_ref(input: &AppState) -> Self {
        input.config.auth_config.clone()
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

pub fn app(config: AppConfig) -> Router {
    let password_checker = Arc::new(PasswordChecker::default());
    let sessions = Arc::new(Sessions::new(config.auth_config.session_expiry));

    Router::new()
        .route("/auth_request", any(auth::handle_auth_request))
        .route(
            &format!("{}/login", config.public_path),
            get(login::handle_get_login).post(login::handle_post_login),
        )
        .with_state(AppState {
            config,
            password_checker,
            sessions,
        })
}
