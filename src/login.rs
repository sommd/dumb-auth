use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Json,
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::{config::Config, cookie, password, sessions::Sessions};

static INDEX_HTML: &str = include_str!("../frontend/index.html");

pub async fn handle_get_login() -> Response {
    Html(INDEX_HTML).into_response()
}

#[derive(Deserialize)]
pub struct LoginForm {
    password: String,
}

pub async fn handle_post_login(
    State(config): State<Config>,
    State(sessions): State<Arc<Sessions>>,
    cookie_jar: CookieJar,
    Json(form): Json<LoginForm>,
) -> Response {
    if !password::check_password(&config, &form.password) {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let session_cookie = cookie::create_session(&config, &sessions).await;

    (cookie_jar.add(session_cookie), StatusCode::OK).into_response()
}
