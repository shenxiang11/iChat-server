use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use async_graphql::{Context, Schema, SimpleObject, Subscription};
use async_graphql::futures_util::Stream;
use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use axum::response::IntoResponse;
use axum::{response, Router};
use axum::extract::State;
use axum::http::{HeaderMap, HeaderName, HeaderValue};
use axum::routing::get;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tower_http::{LatencyUnit, request_id};
use tower_http::request_id::{MakeRequestId, PropagateRequestIdLayer, RequestId};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::{debug, info, Level};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};
use uuid::Uuid;
use crate::app_state::AppState;
use crate::error::AppError;
use crate::middlewares::RequestIdToResponseLayer;
use crate::models::{Chat, Message, UserId};
use crate::query::{MutationRoot, QueryRoot};


pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    async fn interval(
        &self,
        ctx: &Context<'_>,
    ) -> Result<impl Stream<Item = String>, AppError> {
        // let user_id = ctx.data::<UserId>().map_err(|_| AppError::GetGraphqlUserIdError)?;

        let mut rv = ctx.data_unchecked::<Arc<broadcast::Sender<Notification>>>().subscribe();

        Ok(
            async_stream::stream! {
                loop {
                    let noti = rv.recv().await;
                    match noti {
                        Ok(noti) => {
                            let json = serde_json::to_string(noti.event.as_ref()).unwrap();
                            yield json;
                        },
                        Err(e) => {
                            debug!("Error: {:?}", e);
                            break;
                        }
                    }
                }
            }
        )
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

    let schema = Schema::build(QueryRoot::default(), MutationRoot::default(), SubscriptionRoot).data(sender).finish();

    let request_id_header = HeaderName::from_static("ichat-request-id");

    let router = Router::new()
        .route("/", get(graphiql).post(graphql_handler))
        .route_service("/ws", GraphQLSubscription::new(schema.clone()))
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
        .layer(PropagateRequestIdLayer::new(request_id_header))
        .with_state(schema);

    router
}

async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/").subscription_endpoint("/ws").finish())
}

async fn graphql_handler(
    State(schema): State<Schema<QueryRoot, MutationRoot, SubscriptionRoot>>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut  req = req.into_inner();

    let state = AppState::shared().await;

    let token = headers.get("Authorization").map(|v| v.to_str().unwrap_or_default());

    if let Some(token) = token {
        if token.starts_with("Bearer ") {
            let token = token.trim_start_matches("Bearer ");
            let user_id = state.dk.verify(token);

            match user_id {
                Ok(user_id) => {
                    req = req.data::<UserId>(user_id);
                },
                Err(_) => {}
            }
        }
    }

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

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum AppEvent {
    NewChat(Chat),
    AddToChat(Chat),
    RemoveFromChat(Chat),
    NewMessage(Message),
}

#[derive(Debug, Clone)]
pub(crate) struct Notification {
    user_ids: HashSet<u64>,
    event: Arc<AppEvent>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatUpdated {
    op: String,
    old: Option<Chat>,
    new: Option<Chat>,
}

impl Notification {
    pub(crate) fn load(r#type: &str, payload: &str) -> anyhow::Result<Self> {
        let user_ids = HashSet::new();
        let event = match r#type {
            "chat_change" => {
                debug!("Payload: {:?}", payload);
                let payload: ChatUpdated = serde_json::from_str(payload)?;
                info!("Chat updated: {:?}", payload);

                let event = match payload.op.as_str() {
                    "INSERT" => AppEvent::NewChat(payload.new.expect("new should exist")),
                    "UPDATE" => AppEvent::AddToChat(payload.new.expect("new should exist")),
                    "DELETE" => AppEvent::RemoveFromChat(payload.old.expect("old should exist")),
                    _ => return Err(anyhow::anyhow!("Invalid operation")),
                };

                Arc::new(event)
            },
            _ => {
                return Err(anyhow::anyhow!("Invalid event type"));
            }
        };

        Ok(Self { user_ids, event })
    }
}
