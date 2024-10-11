use std::fs;
use std::fs::File;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct AppConfig {
    pub server: ServerConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ServerConfig {
    pub(crate) port: u16,
    pub(crate) postgres_url: String,
    pub(crate) redis_url: String,
}

impl AppConfig {
    pub(crate) fn load() -> anyhow::Result<Self> {
        let config_data = fs::read_to_string("chat_server/ichat.test.toml")?;
        let config: AppConfig = toml::from_str(&config_data)?;
        Ok(config)
    }
}
