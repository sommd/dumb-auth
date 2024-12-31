use std::sync::Arc;

use axum::{
    extract::FromRef,
    routing::{any, get},
    Router,
};

pub use crate::login::LoginForm;
use crate::{config::Config, sessions::Sessions};

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
    config: Config,
    sessions: Arc<Sessions>,
}

impl FromRef<AppState> for Config {
    fn from_ref(input: &AppState) -> Self {
        input.config.clone()
    }
}

impl FromRef<AppState> for Arc<Sessions> {
    fn from_ref(input: &AppState) -> Self {
        input.sessions.clone()
    }
}

pub fn create_app(config: Config) -> Router {
    let sessions = Arc::new(Sessions::new(config.session_expiry));

    Router::new()
        .route("/auth", any(auth::handle_auth_request))
        .route(
            "/login",
            get(login::handle_get_login).post(login::handle_post_login),
        )
        .with_state(AppState { config, sessions })
}
