use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Json,
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::{
    config::{AuthConfig, SessionExpiry},
    passwords::PasswordChecker,
    sessions::SessionStore,
};

static INDEX_HTML: &str = include_str!("../frontend/index.html");

pub async fn handle_get_login() -> Response {
    Html(INDEX_HTML).into_response()
}

#[derive(Deserialize, Serialize)]
pub struct LoginForm {
    pub password: String,
}

pub async fn handle_post_login(
    State(auth_config): State<AuthConfig>,
    State(password_checker): State<Arc<PasswordChecker>>,
    State(session_store): State<Arc<SessionStore>>,
    cookie_jar: CookieJar,
    Json(form): Json<LoginForm>,
) -> Response {
    if !password_checker
        .check_password(&form.password, &auth_config.password)
        .await
    {
        debug!("Login: invalid");
        return StatusCode::UNAUTHORIZED.into_response();
    }

    debug!("Login: valid");

    let (session_token, _) = session_store.create_session().await;
    let session_cookie = create_session_cookie(&auth_config, session_token);

    (cookie_jar.add(session_cookie.into_owned()), StatusCode::OK).into_response()
}

fn create_session_cookie(auth_config: &AuthConfig, session_token: String) -> Cookie<'static> {
    let mut session_cookie =
        Cookie::<'static>::new(auth_config.session_cookie_name.clone(), session_token);

    session_cookie.set_path("/");
    session_cookie.set_same_site(SameSite::Lax);
    session_cookie.set_http_only(true);
    session_cookie.set_secure(true);

    if let Some(domain) = &auth_config.session_cookie_domain {
        session_cookie.set_domain(domain.clone());
    }

    if let SessionExpiry::Duration(expiry) = auth_config.session_expiry {
        session_cookie.set_max_age(expiry);
    }

    session_cookie
}
