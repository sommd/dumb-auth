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

use crate::{
    auth::Authenticator, datastore::DatastoreError, passwords::PasswordChecker,
    sessions::SessionStore,
};

#[cfg(feature = "lmdb")]
pub use crate::datastore::LmdbDatastore;
#[cfg(any(feature = "sqlite", feature = "sqlite-unbundled"))]
pub use crate::datastore::SqliteDatastore;
pub use crate::{
    config::*,
    datastore::{Datastore, InMemoryDatastore},
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

pub fn app(config: AppConfig, datastore: Box<dyn Datastore>) -> Router {
    let password_checker = Arc::new(PasswordChecker::default());
    let session_store = Arc::new(SessionStore::new(
        config.auth_config.session_expiry,
        Arc::from(datastore),
    ));
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
