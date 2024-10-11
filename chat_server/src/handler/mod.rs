mod user;

use axum::Router;
use sqlx::PgPool;
pub(crate) use user::*;
use crate::repository::UserRepository;

pub(crate) async fn init_api_router() -> Router {
    let pool = PgPool::connect("postgres://postgres:postgres@localhost/chat")
        .await
        .expect("Failed to connect to database");

    let redis_client = redis::Client::open("redis://127.0.0.1:6379")
        .expect("Failed to connect to redis");

    let mut conn = redis_client.get_connection()
        .expect("Failed to get redis connection");

    let user_handler = UserHandler::new(pool);

    let api_router = Router::new()
        .nest("/users", user_handler.register_routes());

    let router = Router::new()
        .nest("/api", api_router);

    router
}
