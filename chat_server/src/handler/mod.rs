mod user;

use axum::handler::Handler;
use axum::middleware::from_fn_with_state;
use axum::Router;
use axum::routing::get;

use crate::app_state::AppState;
use crate::middlewares::{verify_token};

pub(crate) async fn init_api_router(state: AppState) -> Router<AppState> {
    let api_router = Router::new()
        .route("/test", get(handle_test))
        .layer(from_fn_with_state(state.clone(), verify_token))
        .nest("/users", user::register_routes());

    let router = Router::new()
        .nest("/api", api_router);

    router
}

async fn handle_test() -> &'static str {
    "Hello, world!"
}
