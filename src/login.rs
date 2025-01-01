use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Json,
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::{config::AuthConfig, cookie, password, sessions::Sessions};

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
    State(sessions): State<Arc<Sessions>>,
    cookie_jar: CookieJar,
    Json(form): Json<LoginForm>,
) -> Response {
    if !password::check_password(&auth_config, &form.password) {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let session_cookie = cookie::create_session(&auth_config, &sessions).await;

    (cookie_jar.add(session_cookie.into_owned()), StatusCode::OK).into_response()
}
