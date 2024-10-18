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
use async_graphql::{Context, EmptyMutation, EmptySubscription, MergedObject, Object, Response, Schema};
use async_graphql::futures_util::SinkExt;
use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQL, GraphQLRequest, GraphQLResponse};
use axum::extract::{Request, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use axum::middleware::{from_fn, from_fn_with_state};
use axum::{Json, response};
use axum::response::IntoResponse;
use axum::routing::get;
use tokio::net::TcpListener;
use tower_http::{LatencyUnit, request_id};
use tower_http::request_id::{MakeRequestId, PropagateRequestId, PropagateRequestIdLayer, RequestId};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::{info, Level, level_filters::LevelFilter};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};
use uuid::Uuid;
use crate::app_state::AppState;
use crate::config::AppConfig;
use crate::error::AppError;
use crate::handler::{init_graphql_router};
use crate::middlewares::{RequestIdToResponseLayer, verify_token};
use crate::models::{Chat, UserId};
use crate::query::{ChatQuery, UserMutation};
use crate::repository::{ChatRepository, UserRepository};

#[derive(MergedObject, Default)]
struct QueryRoot(DemoQuery, ChatQuery);

#[derive(MergedObject, Default)]
struct MutationRoot(UserMutation);

#[derive(Default)]
struct DemoQuery;

#[Object]
impl DemoQuery {
    async fn hello(&self) -> String {
        "hello world".to_string()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = init_graphql_router().await;

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
