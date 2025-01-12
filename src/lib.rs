use std::sync::Arc;

use axum::{
    extract::FromRef,
    routing::{any, get},
    Router,
};
use tower_http::trace::TraceLayer;

use crate::{auth::Authenticator, passwords::PasswordChecker, sessions::SessionStore};
pub use crate::{config::*, login::LoginForm, passwords::hash_password};

mod auth;
mod config;
mod login;
mod passwords;
mod sessions;

#[derive(Clone)]
pub(crate) struct AppState {
    config: AppConfig,
    authenticator: Arc<Authenticator>,
    password_checker: Arc<PasswordChecker>,
    session_store: Arc<SessionStore>,
}

impl FromRef<AppState> for AuthConfig {
    fn from_ref(input: &AppState) -> Self {
        input.config.auth_config.clone()
    }
}

impl FromRef<AppState> for Arc<Authenticator> {
    fn from_ref(input: &AppState) -> Self {
        input.authenticator.clone()
    }
}

impl FromRef<AppState> for Arc<PasswordChecker> {
    fn from_ref(input: &AppState) -> Self {
        input.password_checker.clone()
    }
}

impl FromRef<AppState> for Arc<SessionStore> {
    fn from_ref(input: &AppState) -> Self {
        input.session_store.clone()
    }
}

pub fn app(config: AppConfig) -> Router {
    let password_checker = Arc::new(PasswordChecker::default());
    let session_store = Arc::new(SessionStore::new(config.auth_config.session_expiry));
    let authenticator = Arc::new(Authenticator::new(
        config.public_path.clone(),
        password_checker.clone(),
        session_store.clone(),
    ));

    Router::new()
        .route("/auth_request", any(auth::handle_auth_request))
        .route(
            &format!("{}/login", config.public_path),
            get(login::handle_get_login).post(login::handle_post_login),
        )
        .with_state(AppState {
            config,
            authenticator,
            password_checker,
            session_store,
        })
        .layer(TraceLayer::new_for_http())
}
