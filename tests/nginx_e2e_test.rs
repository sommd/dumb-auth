use std::{
    env,
    process::{Child, Command},
    time::Duration,
};

use axum::http::HeaderValue;
use dumb_auth::{AuthConfig, LoginForm};
use reqwest::{
    header::{self, HeaderMap},
    Client, StatusCode, Url,
};
use serial_test::serial;

use self::common::PASSWORD;

mod common;

const BASE_URI: &str = "http://127.0.0.1:8080";
const TEST_URI: &str = "/index.html?some=param&another=param";

struct Sut {
    base_url: Url,
    dumb_auth: Child,
    nginx: Child,
}

impl Sut {
    pub async fn new(args: &[&str]) -> Self {
        let base_url = Url::parse(BASE_URI).unwrap();

        let mut dumb_auth = Command::new(env!("CARGO_BIN_EXE_dumb-auth"))
            .args(args)
            .spawn()
            .unwrap();
        common::poll_ready("http://127.0.0.1:3862", Duration::from_secs(1)).await;
        assert!(
            dumb_auth.try_wait().unwrap().is_none(),
            "dumb-auth exited unexpectedly"
        );

        let mut nginx = Command::new("nginx")
            .arg("-p")
            .arg(env::current_dir().unwrap().join("examples/nginx"))
            .args(["-c", "nginx.conf"])
            .spawn()
            .unwrap();
        common::poll_ready(BASE_URI, Duration::from_secs(1)).await;
        assert!(
            nginx.try_wait().unwrap().is_none(),
            "nginx exited unexpectedly"
        );

        Self {
            base_url,
            dumb_auth,
            nginx,
        }
    }
}

impl Drop for Sut {
    fn drop(&mut self) {
        Result::and(
            Result::and(self.nginx.kill(), self.dumb_auth.kill()),
            Result::and(self.nginx.wait(), self.dumb_auth.wait()),
        )
        .unwrap();
    }
}

#[tokio::test]
#[serial]
async fn basic_auth() {
    let sut = Sut::new(&[
        "--password",
        PASSWORD,
        "--allow-basic",
        "--allow-session=false",
    ])
    .await;
    let client = Client::new();

    // Make an unauthenticated request
    let res = client
        .get(sut.base_url.join(TEST_URI).unwrap())
        .send()
        .await
        .unwrap();

    // Returns 401
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        res.headers()
            .get_all(header::WWW_AUTHENTICATE)
            .iter()
            .collect::<Vec<_>>(),
        vec![HeaderValue::from_static("Basic realm=\"dumb-auth\"")]
    );

    // Make an authenticated request
    let res = client
        .get(sut.base_url.join(TEST_URI).unwrap())
        .basic_auth("user", Some(PASSWORD))
        .send()
        .await
        .unwrap();

    // Returns 200
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.headers().get(header::WWW_AUTHENTICATE), None);
    assert_eq!(
        res.text().await.unwrap(),
        include_str!("../examples/nginx/www/index.html")
    );
}

#[tokio::test]
#[serial]
async fn bearer_auth() {
    let sut = Sut::new(&[
        "--password",
        PASSWORD,
        "--allow-bearer",
        "--allow-session=false",
    ])
    .await;
    let client = Client::new();

    // Make an unauthenticated request
    let res = client
        .get(sut.base_url.join(TEST_URI).unwrap())
        .send()
        .await
        .unwrap();

    // Returns 401
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        res.headers()
            .get_all(header::WWW_AUTHENTICATE)
            .iter()
            .collect::<Vec<_>>(),
        vec![HeaderValue::from_static("Bearer realm=\"dumb-auth\"")]
    );

    // Make an authenticated request
    let res = client
        .get(sut.base_url.join(TEST_URI).unwrap())
        .bearer_auth(PASSWORD)
        .send()
        .await
        .unwrap();

    // Returns 200
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.headers().get(header::WWW_AUTHENTICATE), None);
    assert_eq!(
        res.text().await.unwrap(),
        include_str!("../examples/nginx/www/index.html")
    );
}

#[tokio::test]
#[serial]
async fn session_auth() {
    let sut = Sut::new(&["--password", PASSWORD]).await;
    let mut headers = HeaderMap::new();
    headers.insert(header::ACCEPT, HeaderValue::from_static("text/html"));
    let client = Client::builder()
        .cookie_store(true)
        .default_headers(headers)
        .build()
        .unwrap();

    // Make an unauthenticated request
    let res = client
        .get(sut.base_url.join(TEST_URI).unwrap())
        .send()
        .await
        .unwrap();

    // Redirects to login
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        res.url(),
        sut.base_url
            .join("/auth/login")
            .unwrap()
            .query_pairs_mut()
            .append_pair("redirect_to", TEST_URI)
            .finish()
    );

    // Do login
    let res = client
        .post(res.url().as_str())
        .json(&LoginForm {
            password: PASSWORD.into(),
        })
        .send()
        .await
        .unwrap();

    // Sets session cookie
    assert_eq!(res.status(), StatusCode::OK);
    assert!(res
        .cookies()
        .find(|c| c.name() == AuthConfig::DEFAULT_SESSION_COOKIE_NAME)
        .is_some());

    // Make now-authenticated request
    let res = client
        .get(sut.base_url.join(TEST_URI).unwrap())
        .send()
        .await
        .unwrap();

    // Returns response without redirecting
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.url(), &sut.base_url.join(TEST_URI).unwrap());
    assert_eq!(
        res.text().await.unwrap(),
        include_str!("../examples/nginx/www/index.html")
    );
}
