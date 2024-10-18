mod error;
mod handler;
mod repository;
mod models;
mod app_state;
mod config;
mod utils;
mod middlewares;
mod query;

use std::net::{Ipv4Addr, SocketAddr};
use anyhow::Result;
use async_graphql::{MergedObject, Object};
use axum::http::HeaderValue;
use tokio::net::TcpListener;
use tower_http::request_id::{MakeRequestId, RequestId};
use tracing::{info};
use uuid::Uuid;

use crate::handler::{init_graphql_router};

#[tokio::main]
async fn main() -> Result<()> {
    let app = init_graphql_router().await;

    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 8080));
    let listener = TcpListener::bind(&address).await?;
    info!("Listening on {address}");
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
