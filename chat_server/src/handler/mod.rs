mod user;

use axum::Router;
pub(crate) use user::*;
use crate::app_state::AppState;

pub(crate) async fn init_api_router() -> Router<AppState> {
    let api_router = Router::new()
        .nest("/users", user::register_routes());

    let router = Router::new()
        .nest("/api", api_router);

    router
}
