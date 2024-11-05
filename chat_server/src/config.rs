use std::ops::Deref;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;

static ONCE: OnceCell<AppConfig> = OnceCell::const_new();

#[derive(Clone)]
pub(crate) struct AppConfig {
    inner: Arc<AppConfigInner>,
}

impl Deref for AppConfig {
    type Target = Arc<AppConfigInner>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct AppConfigInner {
    pub(crate) server: ServerConfig,
    pub(crate) jwt: JwtConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ServerConfig {
    pub(crate) port: u16,
    pub(crate) postgres_url: String,
    pub(crate) redis_url: String,
    pub(crate) request_id_header: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct JwtConfig {
    pub(crate) pk: String,
    pub(crate) sk: String,
    pub(crate) period_seconds: u64,
}

impl AppConfig {
    pub(crate) async fn shared() -> Self {
        ONCE.get_or_init(|| async {
            Self::load()
        }).await.clone()
    }

    pub(crate) fn load() -> Self {
        #[cfg(not(test))]
        let config_data = include_str!("../ichat.test.toml");
        #[cfg(test)]
        let config_data = include_str!("../ichat.unit.test.toml");

        Self {
            inner: Arc::new(
                toml::from_str(&config_data).unwrap()
            )
        }
    }
}
