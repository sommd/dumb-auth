use std::sync::Arc;

use dumb_auth::{AppConfig, AuthConfig, Password};
use reqwest::{cookie, Client, Method, RequestBuilder, Url};
use tokio::{net::TcpListener, task::JoinHandle};

mod basic;
mod bearer;
mod session;

pub const PASSWORD: &str = "hunter2";
pub const ORIGINAL_URI: &str = "/original?uri&query=param";
pub const ORIGINAL_URI_ENCODED: &str = "%2Foriginal%3Furi%26query%3Dparam";

pub struct Sut {
    base_url: Url,
    cookies: Arc<cookie::Jar>,
    client: Client,
    handle: JoinHandle<()>,
}

impl Sut {
    pub async fn default() -> Self {
        Self::with(|_| {}).await
    }

    pub async fn with(configurer: impl FnOnce(&mut AppConfig)) -> Self {
        let mut config = AppConfig::default(AuthConfig::default(Password::Plain(PASSWORD.into())));
        configurer(&mut config);

        Self::new(config).await
    }

    pub async fn new(config: AppConfig) -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();

        let addr = listener.local_addr().unwrap();
        let base_url = Url::parse(&format!("http://{}/", addr)).unwrap();

        let cookies = Arc::new(cookie::Jar::default());
        let client = Client::builder()
            .cookie_provider(cookies.clone())
            .build()
            .unwrap();

        let handle = tokio::spawn(async {
            #[cfg(not(any(feature = "sqlite", feature = "sqlite-unbundled")))]
            let datastore = dumb_auth::InMemoryDatastore::new();
            #[cfg(any(feature = "sqlite", feature = "sqlite-unbundled"))]
            let datastore = dumb_auth::SqliteDatastore::connect(":memory:")
                .await
                .unwrap();

            axum::serve(listener, dumb_auth::app(config, datastore))
                .await
                .unwrap();
        });

        Self {
            base_url,
            cookies,
            client,
            handle,
        }
    }

    pub fn set_cookie(&self, name: &str, value: &str) {
        self.cookies
            .add_cookie_str(&format!("{}={}; Path=/", name, value), &self.base_url);
    }

    pub fn request(&self, method: Method, path: &str) -> RequestBuilder {
        self.client
            .request(method, self.base_url.join(path).unwrap())
    }
}

impl Drop for Sut {
    fn drop(&mut self) {
        self.handle.abort();
    }
}
