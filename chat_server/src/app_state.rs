use std::ops::Deref;
use std::sync::Arc;
use r2d2::Pool;
use r2d2_redis::RedisConnectionManager;
use sqlx::PgPool;
use crate::config::AppConfig;
use crate::error::AppError;
use crate::repository::UserRepository;
use crate::utils::{DecodingKey, EncodingKey};

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) inner: Arc<AppStateInner>,
}

impl AppState {
    pub(crate) async fn try_new(config: AppConfig) -> Result<Self, AppError> {
        let dk = DecodingKey::load(&config.jwt.pk).expect("Failed to load decoding key");
        let ek = EncodingKey::load(&config.jwt.sk).expect("Failed to load encoding key");

        let pool = PgPool::connect(config.server.postgres_url.as_str())
            .await
            .expect("Failed to connect to database");

        let redis_manager = RedisConnectionManager::new(config.server.redis_url.as_str())
            .expect("Failed to create redis connection manager");

        let rdb_pool = Pool::builder().max_size(15).build(redis_manager)
            .expect("Failed to create redis pool");

        Ok(Self {
            inner: Arc::new(AppStateInner {
                config,
                user_repo: UserRepository::new(pool.clone(), rdb_pool.clone()),
                pool,
                rdb_pool,
                dk,
                ek,
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
    pub(crate) config: AppConfig,
    pub(crate) pool: PgPool,
    pub(crate) rdb_pool: Pool<RedisConnectionManager>,
    pub(crate) user_repo: UserRepository,
    pub(crate) dk: DecodingKey,
    pub(crate) ek: EncodingKey,
}
