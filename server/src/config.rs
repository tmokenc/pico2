/**
 * @file config.rs
 * @author Nguyen Le Duy
 * @date 09/04/2025
 * @brief Configuration handling for the server.
 */
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub port: u16,
    pub ip: String,
    pub static_dir: String,
    pub data_dir: String,
    pub sdk_path: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8888,
            ip: String::from("127.0.0.1"),
            static_dir: String::from("./static"),
            data_dir: String::from("./data"),
            sdk_path: None,
        }
    }
}

impl ServerConfig {
    pub fn parse(path: &str) -> Result<Self, config::ConfigError> {
        config::Config::builder()
            .add_source(config::Config::try_from(&ServerConfig::default())?)
            .add_source(config::File::with_name(path).required(false))
            .build()?
            .try_deserialize()
    }
}
