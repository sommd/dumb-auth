use std::sync::Arc;

use dumb_auth::config::AuthConfig;
use reqwest::{cookie, Client, Method, RequestBuilder, Url};
use tokio::{net::TcpListener, task::JoinHandle};

mod basic;
mod bearer;
mod interactive;

pub const PASSWORD: &str = "hunter2";
pub const ORIGINAL_URI: &str = "/original?uri&query=param";
pub const REDIRECT_TO: &str = "%2Foriginal%3Furi%26query%3Dparam";

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

    pub async fn with(auth_configurer: impl FnOnce(&mut AuthConfig)) -> Self {
        let mut auth_config = AuthConfig::new(PASSWORD);
        auth_configurer(&mut auth_config);

        Self::new(auth_config).await
    }

    pub async fn new(auth_config: AuthConfig) -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();

        let addr = listener.local_addr().unwrap();
        let base_url = Url::parse(&format!("http://{}/", addr)).unwrap();

        let cookie_store = Arc::new(cookie::Jar::default());
        let client = Client::builder()
            .cookie_provider(cookie_store.clone())
            .build()
            .unwrap();

        let handle = tokio::spawn(async {
            axum::serve(listener, dumb_auth::create_app(auth_config))
                .await
                .unwrap();
        });

        Self {
            base_url,
            cookies: cookie_store,
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
