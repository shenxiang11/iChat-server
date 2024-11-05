use crate::app_state::AppState;
use crate::error::AppError;
use crate::middlewares::RequestIdToResponseLayer;
use crate::models::{Chat, Message, User, UserId};
use crate::query::{QueryRoot};
use async_graphql::futures_util::Stream;
use async_graphql::http::{
    playground_source, GraphQLPlaygroundConfig, GraphiQLSource, ALL_WEBSOCKET_PROTOCOLS,
};
use async_graphql::{
    Context, Enum, Object, OutputType, Response, Schema, SimpleObject, Subscription, Union,
};
use async_graphql_axum::{
    GraphQLProtocol, GraphQLRequest, GraphQLResponse, GraphQLSubscription, GraphQLWebSocket,
};
use axum::extract::{State, WebSocketUpgrade};
use axum::http::{HeaderMap, HeaderName, HeaderValue};
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::{response, Router};
use serde::{Deserialize, Serialize};
use sqlx::__rt::yield_now;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tower_http::request_id::{MakeRequestId, PropagateRequestIdLayer, RequestId};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tower_http::{request_id, LatencyUnit};
use tower_http::cors::{Any, CorsLayer};
use tracing::level_filters::LevelFilter;
use tracing::{debug, info, Level};
use tracing_subscriber::registry::Data;
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};
use uuid::Uuid;
use crate::mutation::MutationRoot;
use crate::notification::Notification;
use crate::subscription::SubscriptionRoot;

#[derive(Enum, Eq, PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum MutationType {
    Created,
    Deleted,
    Updated,
}

pub(crate) async fn init_graphql_router(app_state: AppState) -> Router {
    let layer = Layer::new().with_filter(LevelFilter::DEBUG);
    tracing_subscriber::registry().with(layer).init();

    let request_id_header = HeaderName::from_static("ichat-request-id");

    let schema = Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        SubscriptionRoot,
    )
        .data(app_state.clone())
        .finish();

    let router = Router::new()
        .route("/", get(graphiql).post(graphql_handler))
        .route("/ws", get(graphql_ws_handler))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Micros),
                ),
        )
        .layer(RequestIdToResponseLayer::new(request_id_header.clone()))
        .layer(request_id::SetRequestIdLayer::new(
            request_id_header.clone(),
            RequestIdGenerator,
        ))
        .layer(PropagateRequestIdLayer::new(request_id_header))
        .with_state(schema);

    router
}

async fn graphiql() -> impl IntoResponse {
    Html(
        GraphiQLSource::build()
            .endpoint("/")
            .subscription_endpoint("/ws")
            .finish(),
    )
}

async fn graphql_ws_handler(
    State(schema): State<Schema<QueryRoot, MutationRoot, SubscriptionRoot>>,
    protocol: GraphQLProtocol,
    websocket: WebSocketUpgrade,
) -> response::Response {
    websocket
        .protocols(ALL_WEBSOCKET_PROTOCOLS)
        .on_upgrade(move |stream| {
            GraphQLWebSocket::new(stream, schema.clone(), protocol)
                .on_connection_init(|x| async move {
                    handle_connect_init(x).await
                })
                .serve()
        })
}

pub async fn get_user_id_from_bearer_token(str: Option<&str>) -> Option<UserId> {
    Some(1)
    // if let Some(token) = str {
    //     if token.starts_with("Bearer ") {
    //         let token = token.trim_start_matches("Bearer ");
    //         let user_id = state.dk.verify(token);
    //
    //         match user_id {
    //             Ok(user_id) => Some(user_id),
    //             Err(_) => None,
    //         }
    //     } else {
    //         None
    //     }
    // } else {
    //     None
    // }
}

pub async fn handle_connect_init(
    value: serde_json::Value,
) -> async_graphql::Result<async_graphql::Data> {
    let bearer_token_str = value
        .get("Authorization")
        .map(|v| v.as_str().unwrap_or_default());

    let user_id = get_user_id_from_bearer_token(bearer_token_str).await;

    let mut data = async_graphql::Data::default();

    if let Some(user_id) = user_id {
        data.insert(user_id);
        Ok(data)
    } else {
        Err(AppError::Unauthorized.into())
    }
}

async fn graphql_handler(
    State(schema): State<Schema<QueryRoot, MutationRoot, SubscriptionRoot>>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut req = req.into_inner();

    let token = headers
        .get("Authorization")
        .map(|v| v.to_str().unwrap_or_default());
    debug!("Bearer token: {:?}", token);

    let user_id = get_user_id_from_bearer_token(token).await;

    if let Some(user_id) = user_id {
        req = req.data::<UserId>(user_id);
    }
    // FIXME: 这里没有授权的情况下，没有立刻终止
    // 因为我也不知道怎么返回错误
    // 但是如果需要授权的请求需要用到 userId，则一定会报错的，所以并没有太大的影响

    schema.execute(req).await.into()
}

#[derive(Debug, Clone)]
struct RequestIdGenerator;

impl MakeRequestId for RequestIdGenerator {
    fn make_request_id<B>(&mut self, _request: &axum::http::Request<B>) -> Option<RequestId> {
        let request_id = Uuid::now_v7().to_string();
        HeaderValue::from_str(&request_id).ok().map(RequestId::from)
    }
}
