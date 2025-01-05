use dumb_auth::LoginForm;
use reqwest::{header, Method, StatusCode};

use super::*;

const SESSION_COOKIE_NAME: &str = "dumb-auth-session";

#[tokio::test]
async fn redirects_to_login_when_no_session() {
    let res = Sut::default()
        .await
        .request(Method::GET, "/auth")
        .header("X-Original-URI", ORIGINAL_URI)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        res.headers().get(header::LOCATION).unwrap(),
        &format!("/login?redirect_to={}", REDIRECT_TO)
    );
}

#[tokio::test]
async fn redirects_to_login_when_session_invalid() {
    let sut = Sut::default().await;
    sut.set_cookie(SESSION_COOKIE_NAME, "invalid");

    let res = sut
        .request(Method::GET, "/auth")
        .header("X-Original-URI", ORIGINAL_URI)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        res.headers().get(header::LOCATION).unwrap(),
        &format!("/login?redirect_to={}", REDIRECT_TO)
    );
}

#[tokio::test]
async fn login_returns_401_with_incorrect_password() {
    let res = Sut::default()
        .await
        .request(Method::POST, "/login")
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
        .request(Method::POST, "/login")
        .json(&LoginForm {
            password: PASSWORD.into(),
        })
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert!(res
        .cookies()
        .find(|c| c.name() == SESSION_COOKIE_NAME)
        .is_some());

    let res = sut
        .request(Method::GET, "/auth")
        .header("X-Original-URI", ORIGINAL_URI)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}
