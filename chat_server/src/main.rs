mod error;
mod handler;
mod repository;
mod models;

use std::net::{Ipv4Addr, SocketAddr};
use anyhow::Result;
use axum::response::IntoResponse;
use axum::{Router, ServiceExt};
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

use crate::handler::{init_api_router, UserHandler};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::DEBUG);
    tracing_subscriber::registry().with(layer).init();

    let app = init_api_router();

    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 8080));
    let listener = TcpListener::bind(&address).await?;
    info!("Listening on {address}");
    axum::serve(listener, app.await.into_make_service()).await?;

    Ok(())
}
