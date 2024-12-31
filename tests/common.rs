use std::net::SocketAddr;

use dumb_auth::config::Config;
use tokio::{net::TcpListener, spawn};
use tokio_util::task::AbortOnDropHandle;

pub async fn start_dumb_auth(config: Config) -> (SocketAddr, impl Drop) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let handle = spawn(async {
        axum::serve(listener, dumb_auth::create_app(config))
            .await
            .unwrap();
    });

    (addr, AbortOnDropHandle::new(handle))
}
