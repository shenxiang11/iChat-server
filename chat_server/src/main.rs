mod error;
mod handler;
mod repository;
mod models;
mod app_state;
mod config;
mod utils;
mod middlewares;

use std::net::{Ipv4Addr, SocketAddr};
use anyhow::Result;
use axum::extract::Request;
use axum::http::{HeaderName, HeaderValue};
use axum::middleware::from_fn;
use tokio::net::TcpListener;
use tower_http::{LatencyUnit, request_id};
use tower_http::request_id::{MakeRequestId, PropagateRequestId, PropagateRequestIdLayer, RequestId};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::{info, Level, level_filters::LevelFilter};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};
use uuid::Uuid;
use crate::app_state::AppState;
use crate::config::AppConfig;
use crate::handler::{init_api_router};
use crate::middlewares::{RequestIdToResponseLayer};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::DEBUG);
    tracing_subscriber::registry().with(layer).init();

    let config = AppConfig::load()?;

    let state = AppState::try_new(config).await?;

    let request_id_header = HeaderName::from_static("ichat-request-id");

    let app = init_api_router(state.clone()).await;
    let app = app.with_state(state.clone());
    let app = app
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Micros),
                )
        )
        .layer(RequestIdToResponseLayer::new(request_id_header.clone()))
        .layer(request_id::SetRequestIdLayer::new(request_id_header.clone(), RequestIdGenerator))
        .layer(PropagateRequestIdLayer::new(request_id_header));

    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 8080));
    let listener = TcpListener::bind(&address).await?;
    info!("Listening on {address}");
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

#[derive(Debug, Clone)]
struct RequestIdGenerator;

impl MakeRequestId for RequestIdGenerator {
    fn make_request_id<B>(&mut self, _request: &axum::http::Request<B>) -> Option<RequestId> {
        let request_id = Uuid::now_v7().to_string();
        HeaderValue::from_str(&request_id).ok().map(RequestId::from)
    }
}
