use dumb_auth::{config::AuthConfig, LoginForm};
use reqwest::{header, Client, StatusCode};

use self::common::start_dumb_auth;

mod common;

const PASSWORD: &str = "hunter2";
const ORIGINAL_URI: &str = "/original?query&params";

// Interactive/session auth

#[tokio::test]
async fn auth_redirects_to_login_when_session_auth_fails() {
    let (addr, _handle) = start_dumb_auth(AuthConfig::new(PASSWORD)).await;

    let res = Client::new()
        .get(format!("http://{}/auth", addr))
        .header("X-Original-URI", ORIGINAL_URI)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        res.headers().get(header::LOCATION).unwrap(),
        "/login?redirect_to=%2Foriginal%3Fquery%26params"
    );
}

#[tokio::test]
async fn login_does_not_grant_session_when_interactive_auth_fails() {
    let (addr, _handle) = start_dumb_auth(AuthConfig::new(PASSWORD)).await;

    let client = Client::builder().cookie_store(true).build().unwrap();

    let res = client
        .post(format!("http://{}/login", addr))
        .json(&LoginForm {
            password: "incorrect".into(),
        })
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    let res = client
        .get(format!("http://{}/auth", addr))
        .header("X-Original-URI", ORIGINAL_URI)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_grants_session_when_interactive_auth_succeeds() {
    let (addr, _handle) = start_dumb_auth(AuthConfig::new(PASSWORD)).await;

    let client = Client::builder().cookie_store(true).build().unwrap();

    let res = client
        .post(format!("http://{}/login", addr))
        .json(&LoginForm {
            password: PASSWORD.into(),
        })
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = client
        .get(format!("http://{}/auth", addr))
        .header("X-Original-URI", ORIGINAL_URI)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

// Basic auth

#[tokio::test]
async fn auth_returns_401_when_basic_auth_fails() {
    let (addr, _handle) = start_dumb_auth(AuthConfig {
        allow_basic: true,
        ..AuthConfig::new(PASSWORD)
    })
    .await;

    let res = Client::new()
        .get(format!("http://{}/auth", addr))
        .header("X-Original-URI", ORIGINAL_URI)
        .basic_auth("user", None::<&str>)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(res.headers().get(header::LOCATION), None);
}

#[tokio::test]
async fn auth_returns_200_when_basic_auth_succeeds() {
    let (addr, _handle) = start_dumb_auth(AuthConfig {
        allow_basic: true,
        ..AuthConfig::new(PASSWORD)
    })
    .await;

    let res = Client::new()
        .get(format!("http://{}/auth", addr))
        .header("X-Original-URI", ORIGINAL_URI)
        .basic_auth("user", Some(PASSWORD))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}

// Bearer auth

#[tokio::test]
async fn auth_returns_401_when_bearer_auth_fails() {
    let (addr, _handle) = start_dumb_auth(AuthConfig {
        allow_bearer: true,
        ..AuthConfig::new(PASSWORD)
    })
    .await;

    let res = Client::new()
        .get(format!("http://{}/auth", addr))
        .header("X-Original-URI", ORIGINAL_URI)
        .bearer_auth("invalid")
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(res.headers().get(header::LOCATION), None);
}

#[tokio::test]
async fn auth_returns_200_when_bearer_auth_succeeds() {
    let (addr, _handle) = start_dumb_auth(AuthConfig {
        allow_bearer: true,
        ..AuthConfig::new(PASSWORD)
    })
    .await;

    let res = Client::new()
        .get(format!("http://{}/auth", addr))
        .header("X-Original-URI", ORIGINAL_URI)
        .bearer_auth(PASSWORD)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}
