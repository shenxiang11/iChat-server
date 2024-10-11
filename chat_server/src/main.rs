mod error;
mod handler;
mod repository;
mod models;
mod app_state;
mod config;
mod utils;

use std::net::{Ipv4Addr, SocketAddr};
use anyhow::Result;
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

use crate::app_state::AppState;
use crate::config::AppConfig;
use crate::handler::{init_api_router};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::DEBUG);
    tracing_subscriber::registry().with(layer).init();

    let config = AppConfig::load()?;

    let state = AppState::try_new(config).await?;

    let app = init_api_router().await;
    let app = app.with_state(state.clone());

    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 8080));
    let listener = TcpListener::bind(&address).await?;
    info!("Listening on {address}");
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
