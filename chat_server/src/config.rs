use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct AppConfig {
    pub(crate) server: ServerConfig,
    pub(crate) jwt: JwtConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ServerConfig {
    pub(crate) port: u16,
    pub(crate) postgres_url: String,
    pub(crate) redis_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct JwtConfig {
    pub(crate) pk: String,
    pub(crate) sk: String,
}

impl AppConfig {
    pub(crate) fn load() -> anyhow::Result<Self> {
        let config_data = fs::read_to_string("chat_server/ichat.test.toml")?;
        let config: AppConfig = toml::from_str(&config_data)?;
        Ok(config)
    }
}
