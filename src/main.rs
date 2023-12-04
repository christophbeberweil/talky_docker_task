use std::net::SocketAddr;

use crate::config::Config;
use crate::views::render_folder_contents;
use axum::{routing::get, Router};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use types::TalkyError;
mod config;
mod types;
mod util;
mod views;
#[tokio::main]
async fn main() -> Result<(), TalkyError> {
    let config = Config::init();
    let port: u16 = config.app_port;

    // init tracing setup

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_templates=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    /*tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    */
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("🚀 listening on {}, serving {}", &addr, &config.base_dir);

    let router = Router::new()
        .route("/", get(render_folder_contents))
        .fallback(get(render_folder_contents))
        .with_state(config);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router).await.unwrap();

    Ok(())
}
