use std::{
    process::{Child, Command},
    time::Duration,
};

use dumb_auth::LoginForm;
use reqwest::{Client, StatusCode, Url};

use common::PASSWORD;

mod common;

const SESSION_COOKIE_NAME: &str = "dumb-auth-session";
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

        let dumb_auth = Command::new(env!("CARGO_BIN_EXE_dumb-auth"))
            .args(args)
            .spawn()
            .unwrap();
        common::poll_ready("http://127.0.0.1:3862", Duration::from_secs(1)).await;

        let nginx = Command::new("nginx")
            .args(&[
                "-p",
                concat!(env!("CARGO_MANIFEST_DIR"), "/examples/nginx/"),
            ])
            .args(["-c", "nginx.conf"])
            .spawn()
            .unwrap();
        common::poll_ready(BASE_URI, Duration::from_secs(1)).await;

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
async fn interactive_auth() {
    let sut = Sut::new(&["--password", PASSWORD]).await;
    let client = Client::builder().cookie_store(true).build().unwrap();

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
            .join("/login")
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
        .find(|c| c.name() == SESSION_COOKIE_NAME)
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
