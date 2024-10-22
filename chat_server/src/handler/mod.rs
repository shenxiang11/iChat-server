use crate::app_state::AppState;
use crate::error::AppError;
use crate::middlewares::RequestIdToResponseLayer;
use crate::models::{Chat, Message, User, UserId};
use crate::query::{MutationRoot, QueryRoot};
use async_graphql::futures_util::Stream;
use async_graphql::http::{
    playground_source, GraphQLPlaygroundConfig, GraphiQLSource, ALL_WEBSOCKET_PROTOCOLS,
};
use async_graphql::{ComplexObject, Context, Enum, Object, OutputType, Response, Schema, SimpleObject, Subscription, Union};
use async_graphql_axum::{
    GraphQLProtocol, GraphQLRequest, GraphQLResponse, GraphQLSubscription, GraphQLWebSocket,
};
use axum::extract::{State, WebSocketUpgrade};
use axum::http::{HeaderMap, HeaderName, HeaderValue};
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::{response, Router};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use sqlx::__rt::yield_now;
use tokio::sync::broadcast;
use tower_http::request_id::{MakeRequestId, PropagateRequestIdLayer, RequestId};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tower_http::{request_id, LatencyUnit};
use tracing::level_filters::LevelFilter;
use tracing::{debug, info, Level};
use tracing_subscriber::registry::Data;
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};
use uuid::Uuid;

#[derive(Enum, Eq, PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum MutationType {
    Created,
    Deleted,
    Updated,
}

pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    async fn chat(
        &self,
        ctx: &Context<'_>,
    ) -> Result<impl Stream<Item = AppEvent>, AppError> {
        let user_id = ctx
            .data::<UserId>()
            .map_err(|_| AppError::GetGraphqlUserIdError)?;
        let state = AppState::shared().await;

        let mut rv = ctx
            .data_unchecked::<Arc<broadcast::Sender<Notification>>>()
            .subscribe();

        Ok(async_stream::stream! {
            loop {
                let noti = rv.recv().await;
                match noti {
                    Ok(noti) => {
                        yield noti.event;
                    },
                    Err(e) => {
                        debug!("Error: {:?}", e);
                    }
                }
            }
        })
    }

    async fn interval2(&self, #[graphql(default = 1)] n: i32) -> impl Stream<Item = i32> {
        let mut value = 0;
        debug!("Init Stream value: {}", value);
        async_stream::stream! {
            loop {
                futures_timer::Delay::new(Duration::from_secs(1)).await;
                value += n;
                debug!("Stream value: {}", value);
                yield value;
            }
        }
    }
}

pub(crate) async fn init_graphql_router(sender: Arc<broadcast::Sender<Notification>>) -> Router {
    let layer = Layer::new().with_filter(LevelFilter::DEBUG);
    tracing_subscriber::registry().with(layer).init();

    let schema = Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        SubscriptionRoot,
    )
    .data(sender)
    .finish();

    let request_id_header = HeaderName::from_static("ichat-request-id");

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
                .on_connection_init(handle_connect_init)
                .serve()
        })
}

pub async fn get_user_id_from_bearer_token(str: Option<&str>) -> Option<UserId> {
    let state = AppState::shared().await;

    if let Some(token) = str {
        if token.starts_with("Bearer ") {
            let token = token.trim_start_matches("Bearer ");
            let user_id = state.dk.verify(token);

            match user_id {
                Ok(user_id) => Some(user_id),
                Err(_) => None,
            }
        } else {
            None
        }
    } else {
        None
    }
}

pub async fn handle_connect_init(
    value: serde_json::Value,
) -> async_graphql::Result<async_graphql::Data> {
    let bearer_token_str = value
        .get("Authorization")
        .map(|v| v.as_str().unwrap_or_default());
    debug!("Bearer token: {:?}", bearer_token_str);

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

#[derive(Debug, Serialize, Deserialize, Union, Clone)]
#[serde(tag = "event")]
pub enum AppEvent {
    CreatedChat(CreatedChat),
    ChatOwnerChanged(ChatOwnerChanged),
    ChatNameChanged(ChatNameChanged),
    ChatDeleted(ChatDeleted),
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
struct CreatedChat {
    data: Chat,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
struct ChatOwnerChanged {
    data: Chat,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
struct ChatNameChanged {
    data: Chat,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
struct ChatDeleted {
    data: Chat,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ChatEvent {
    NewChat(Chat),
    ChatOwnerChanged(Chat),
    ChatNameChanged(Chat),
    ChatDeleted(Chat),
}

#[derive(Debug, Clone)]
pub(crate) struct Notification {
    event: AppEvent,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatUpdated {
    op: String,
    old: Option<Chat>,
    new: Option<Chat>,
}

#[derive(Debug, Serialize, Deserialize, SimpleObject)]
struct SubscriptionPayload<T>
where
    T: Serialize + Send + Sync + OutputType,
{
    mutation_type: MutationType,
    data: T,
}

impl Notification {
    pub(crate) fn load(r#type: &str, payload: &str) -> Result<Self, AppError> {
        let event = match r#type {
            "chat_change" => Self::handle_chat_change(payload)?,
            _ => {
                return Err(AppError::NotificationError("Invalid operation".to_string()));
            }
        };

        Ok(Self { event })
    }

    pub(crate) fn handle_chat_change(payload: &str) -> Result<AppEvent, AppError> {
        let payload: ChatUpdated = serde_json::from_str(payload)?;

        let event = match payload.op.as_str() {
            "INSERT" => match payload.new {
                Some(new) => AppEvent::CreatedChat(CreatedChat { data: new }),
                None => {
                    return Err(AppError::NotificationError("Invalid operation".to_string()));
                }
            },
            "UPDATE" => {
                match (payload.old, payload.new) {
                    (Some(old), Some(new)) => {
                        if old.owner_id != new.owner_id {
                            AppEvent::ChatOwnerChanged(ChatOwnerChanged { data: new })
                        } else if old.name != new.name {
                            AppEvent::ChatNameChanged(ChatNameChanged { data: new })
                        } else {
                            return Err(AppError::NotificationError(
                                "Invalid operation".to_string(),
                            ));
                        }
                    }
                    _ => {
                        return Err(AppError::NotificationError("Invalid operation".to_string()));
                    }
                }
            }
            "DELETE" => match payload.old {
                Some(old) => AppEvent::ChatDeleted(ChatDeleted { data: old }),
                None => {
                    return Err(AppError::NotificationError("Invalid operation".to_string()));
                }
            },
            _ => {
                return Err(AppError::NotificationError("Invalid operation".to_string()));
            }
        };

        Ok(event)
    }
}
