use dumb_auth::{AuthConfig, LoginForm};
use reqwest::{header, Method, StatusCode};

use super::{Sut, ORIGINAL_URI, ORIGINAL_URI_ENCODED, PASSWORD};

#[tokio::test]
async fn redirects_browser_to_login_when_no_session() {
    let res = Sut::default()
        .await
        .request(Method::GET, "/auth_request")
        .header(header::ACCEPT, "text/html")
        .header("X-Original-URI", ORIGINAL_URI)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        res.headers().get(header::LOCATION).unwrap(),
        &format!("/auth/login?redirect_to={}", ORIGINAL_URI_ENCODED)
    );
    assert_eq!(res.headers().get(header::WWW_AUTHENTICATE), None);
}

#[tokio::test]
async fn redirects_browser_to_login_when_session_invalid() {
    let sut = Sut::default().await;
    sut.set_cookie(AuthConfig::DEFAULT_SESSION_COOKIE_NAME, "invalid");

    let res = sut
        .request(Method::GET, "/auth_request")
        .header(header::ACCEPT, "text/html")
        .header("X-Original-URI", ORIGINAL_URI)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        res.headers().get(header::LOCATION).unwrap(),
        &format!("/auth/login?redirect_to={}", ORIGINAL_URI_ENCODED)
    );
    assert_eq!(res.headers().get(header::WWW_AUTHENTICATE), None);
}

#[tokio::test]
async fn returns_401_when_non_browser() {
    let res = Sut::default()
        .await
        .request(Method::GET, "/auth_request")
        .header("X-Original-URI", ORIGINAL_URI)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(res.headers().get(header::LOCATION), None);
    assert_eq!(res.headers().get(header::WWW_AUTHENTICATE), None);
}

#[tokio::test]
async fn login_returns_401_with_incorrect_password() {
    let res = Sut::default()
        .await
        .request(Method::POST, "/auth/login")
        .json(&LoginForm {
            password: "invalid".into(),
        })
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(res.cookies().count(), 0);
}

#[tokio::test]
async fn login_grants_session_with_correct_password() {
    let sut = Sut::default().await;

    let res = sut
        .request(Method::POST, "/auth/login")
        .json(&LoginForm {
            password: PASSWORD.into(),
        })
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert!(res
        .cookies()
        .find(|c| c.name() == AuthConfig::DEFAULT_SESSION_COOKIE_NAME)
        .is_some());

    let res = sut
        .request(Method::GET, "/auth_request")
        .header("X-Original-URI", ORIGINAL_URI)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.headers().get(header::LOCATION), None);
    assert_eq!(res.headers().get(header::WWW_AUTHENTICATE), None);
}
