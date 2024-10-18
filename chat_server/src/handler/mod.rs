use async_graphql::{EmptySubscription, Schema};
use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::response::IntoResponse;
use axum::{response, Router};
use axum::extract::State;
use axum::http::{HeaderMap, HeaderName, HeaderValue};
use axum::routing::get;
use tower_http::{LatencyUnit, request_id};
use tower_http::request_id::{MakeRequestId, PropagateRequestIdLayer, RequestId};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::middlewares::RequestIdToResponseLayer;
use crate::models::UserId;
use crate::query::{MutationRoot, QueryRoot};

pub(crate) async fn init_graphql_router() -> Router {
    let layer = Layer::new().with_filter(LevelFilter::DEBUG);
    tracing_subscriber::registry().with(layer).init();

    let schema = Schema::build(QueryRoot::default(), MutationRoot::default(), EmptySubscription).finish();

    let request_id_header = HeaderName::from_static("ichat-request-id");

    let router = Router::new()
        .route("/", get(graphiql).post(graphql_handler))
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
    response::Html(GraphiQLSource::build().endpoint("/").finish())
}

async fn graphql_handler(
    State(schema): State<Schema<QueryRoot, MutationRoot, EmptySubscription>>,
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
