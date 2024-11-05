mod error;
mod handler;
mod repository;
mod models;
mod app_state;
mod config;
mod utils;
mod middlewares;
mod query;
mod mutation;
mod subscription;
mod notification;

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use anyhow::Result;
use async_graphql::{MergedObject, Object};
use async_graphql::futures_util::StreamExt;
use axum::http::HeaderValue;
use sqlx::postgres::PgListener;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tower_http::request_id::{MakeRequestId, RequestId};
use tracing::{error, info};
use crate::config::AppConfig;
use crate::handler::{init_graphql_router};
use crate::notification::setup_pg_listener;

#[tokio::main]
async fn main() -> Result<()> {
    let (sender, _) = broadcast::channel(16);
    let sender = Arc::new(sender);

    let app = init_graphql_router(sender.clone()).await;

    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 8080));
    let listener = TcpListener::bind(&address).await?;
    info!("Listening on {address}");

    tokio::spawn(setup_pg_listener(sender.clone()));

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
