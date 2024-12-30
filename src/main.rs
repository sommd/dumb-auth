use clap::Parser;
use tokio::net::TcpListener;

use crate::config::Config;

mod app;
mod auth;
mod basic;
mod bearer;
mod config;
mod cookie;
mod login;
mod password;
mod sessions;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let config = Config::parse();
    let listener = TcpListener::bind(&config.bind_addr).await.unwrap();
    let app = app::create(config);
    axum::serve(listener, app).await.unwrap();
}
