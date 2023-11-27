use std::net::SocketAddr;

use crate::config::Config;
use crate::views::render_folder_contents;
use axum::{routing::get, Router};
use types::TalkyError;
mod config;
mod types;
mod views;

#[tokio::main]
async fn main() -> Result<(), TalkyError> {
    let config = Config::init();
    let port: u16 = config.app_port;

    let router = Router::new()
        .route("/", get(render_folder_contents))
        .fallback(get(render_folder_contents))
        .with_state(config);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("🚀 listening on {}", &addr);

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .expect("that the server could be started...");
    Ok(())
}
