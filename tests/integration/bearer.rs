use reqwest::{Method, StatusCode};

use super::{Sut, ORIGINAL_URI, PASSWORD};

async fn sut() -> Sut {
    Sut::with(|c| {
        c.allow_bearer = true;
        c.allow_session = false;
    })
    .await
}

#[tokio::test]
async fn returns_401_with_no_auth() {
    let res = sut()
        .await
        .request(Method::GET, "/auth")
        .header("X-Original-URI", ORIGINAL_URI)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn returns_401_with_incorrect_password() {
    let sut = Sut::with(|c| {
        c.allow_basic = true;
        c.allow_session = false;
    })
    .await;

    let res = sut
        .request(Method::GET, "/auth")
        .header("X-Original-URI", ORIGINAL_URI)
        .bearer_auth("invalid")
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn returns_200_with_correct_password() {
    let res = sut()
        .await
        .request(Method::GET, "/auth")
        .header("X-Original-URI", ORIGINAL_URI)
        .bearer_auth(PASSWORD)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}
