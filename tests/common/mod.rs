use std::{ops::Div, time::Duration};

use reqwest::Client;
use tokio::time;

pub const PASSWORD: &str = "hunter2";

pub async fn poll_ready(url: &str, timeout: Duration) {
    time::timeout(timeout, async {
        let client = Client::builder().timeout(timeout).build().unwrap();
        let interval = timeout.div(10).max(Duration::from_millis(50));

        loop {
            if client.get(url).send().await.is_ok() {
                break;
            }

            time::sleep(interval).await;
        }
    })
    .await
    .unwrap();
}
