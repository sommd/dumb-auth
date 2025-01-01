use axum::http::HeaderMap;
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};

use crate::{
    auth::AuthResult,
    config::{AuthConfig, SessionExpiry},
    sessions::Sessions,
};

pub async fn validate_session(
    auth_config: &AuthConfig,
    sessions: &Sessions,
    headers: &HeaderMap,
) -> AuthResult {
    if let Some(cookie) = CookieJar::from_headers(headers).get(&auth_config.session_cookie_name) {
        if sessions.get_valid_session(cookie.value()).await.is_some() {
            AuthResult::Valid
        } else {
            AuthResult::Invalid
        }
    } else {
        AuthResult::Missing
    }
}

pub async fn create_session(auth_config: &AuthConfig, sessions: &Sessions) -> Cookie<'static> {
    let (session_token, _) = sessions.create_session().await;

    let mut session_cookie = Cookie::new(auth_config.session_cookie_name.clone(), session_token);
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
