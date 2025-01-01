use std::sync::Arc;

use axum::{
    extract::FromRef,
    routing::{any, get},
    Router,
};

use crate::{config::AuthConfig, sessions::Sessions};

pub use crate::login::LoginForm;

mod auth;
mod basic;
mod bearer;
pub mod config;
mod cookie;
mod login;
mod password;
mod sessions;

#[derive(Clone)]
pub(crate) struct AppState {
    auth_config: AuthConfig,
    sessions: Arc<Sessions>,
}

impl FromRef<AppState> for AuthConfig {
    fn from_ref(input: &AppState) -> Self {
        input.auth_config.clone()
    }
}

impl FromRef<AppState> for Arc<Sessions> {
    fn from_ref(input: &AppState) -> Self {
        input.sessions.clone()
    }
}

pub fn create_app(auth_config: AuthConfig) -> Router {
    let sessions = Arc::new(Sessions::new(auth_config.session_expiry));

    Router::new()
        .route("/auth", any(auth::handle_auth_request))
        .route(
            "/login",
            get(login::handle_get_login).post(login::handle_post_login),
        )
        .with_state(AppState {
            auth_config,
            sessions,
        })
}
