use std::fmt::{Debug};
use std::ops::Deref;
use std::sync::Arc;
use r2d2::Pool;
use r2d2_redis::RedisConnectionManager;
use sqlx::PgPool;
use crate::error::AppError;
use crate::repository::UserRepository;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) inner: Arc<AppStateInner>,
}

impl AppState {
    pub(crate) async fn try_new() -> Result<Self, AppError> {
        let pool = PgPool::connect("postgres://postgres:postgres@localhost/chat")
            .await
            .expect("Failed to connect to database");

        let redis_manager = RedisConnectionManager::new("redis://127.0.0.1:6379")
            .expect("Failed to create redis connection manager");

        let rdb_pool = Pool::builder().max_size(15).build(redis_manager)
            .expect("Failed to create redis pool");

        Ok(Self {
            inner: Arc::new(AppStateInner {
                user_repo: UserRepository::new(pool.clone(), rdb_pool.clone()),
                pool,
                rdb_pool,
            }),
        })
    }
}

impl Deref for AppState {
    type Target = Arc<AppStateInner>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub(crate) struct AppStateInner {
    pub(crate) pool: PgPool,
    pub(crate) rdb_pool: Pool<RedisConnectionManager>,
    pub(crate) user_repo: UserRepository,
}
