use std::sync::Arc;

use axum::{
    extract::FromRef,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{any, get},
    Router,
};
use thiserror::Error;
use tower_http::trace::TraceLayer;
use tracing::error;

use crate::{auth::Authenticator, passwords::PasswordChecker, sessions::SessionManager};

pub use crate::{
    config::*,
    datastore::{Datastore, DatastoreError, ReadMode, WriteMode},
    login::LoginForm,
    passwords::hash_password,
};

mod auth;
mod config;
mod datastore;
mod login;
mod passwords;
mod sessions;

#[derive(Clone)]
struct AppState {
    config: AppConfig,
    authenticator: Arc<Authenticator>,
    password_checker: Arc<PasswordChecker>,
    session_manager: Arc<SessionManager>,
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

impl FromRef<AppState> for Arc<SessionManager> {
    fn from_ref(input: &AppState) -> Self {
        input.session_manager.clone()
    }
}

#[derive(Debug, Error)]
enum AppError {
    #[error("{0}")]
    DatastoreError(#[from] DatastoreError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::DatastoreError(e) => {
                error!("Error from datastore: {e}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        status.into_response()
    }
}

pub fn app(config: AppConfig, datastore: Datastore) -> Router {
    let password_checker = Arc::new(PasswordChecker::default());
    let session_manager = Arc::new(SessionManager::new(
        config.auth_config.session_expiry,
        Arc::from(datastore),
    ));
    let authenticator = Arc::new(Authenticator::new(
        config.public_path.clone(),
        password_checker.clone(),
        session_manager.clone(),
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
            session_manager,
        })
        .layer(TraceLayer::new_for_http())
}
