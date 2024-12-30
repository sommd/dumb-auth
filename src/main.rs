use clap::Parser;
use tokio::net::TcpListener;

use dumb_auth::config::Config;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let config = Config::parse();
    let listener = TcpListener::bind(&config.bind_addr).await.unwrap();
    let app = dumb_auth::create_app(config);
    axum::serve(listener, app).await.unwrap();
}
