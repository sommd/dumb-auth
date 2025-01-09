use dumb_auth::AppConfig;
use reqwest::{Method, StatusCode};

use super::{Sut, ORIGINAL_URI, PASSWORD};

fn configure(config: &mut AppConfig) {
    config.auth_config.allow_bearer = true;
    config.auth_config.allow_session = false;
}

#[tokio::test]
async fn returns_401_with_no_auth() {
    let res = Sut::with(configure)
        .await
        .request(Method::GET, "/auth_request")
        .header("X-Original-URI", ORIGINAL_URI)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn returns_401_with_incorrect_password() {
    let res = Sut::with(configure)
        .await
        .request(Method::GET, "/auth_request")
        .header("X-Original-URI", ORIGINAL_URI)
        .bearer_auth("invalid")
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn returns_200_with_correct_password() {
    let res = Sut::with(configure)
        .await
        .request(Method::GET, "/auth_request")
        .header("X-Original-URI", ORIGINAL_URI)
        .bearer_auth(PASSWORD)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}
