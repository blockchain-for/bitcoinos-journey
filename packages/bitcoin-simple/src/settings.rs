use config::{Config, Environment};
use serde::{Deserialize, Serialize};

pub const CONFIG_NAME: &str = "config";
pub const CONFIG_FILE_NAME: &str = "config.yaml";
pub const ENV_PREFIX: &str = "BITCOIN";

const SETTINGS_SEPARATOR: &str = "__";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Settings {
    pub config: ConfigInfo,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConfigInfo {
    pub rpc_port: u32,
    pub tcp_port: u32,
    pub web_port: u32,
    pub data_dir: String,
    pub miner_enabled: bool,
    pub bootstrap_nodes: Vec<String>,
}

impl Settings {
    pub fn new(location: &str, env_prefix: &str) -> Result<Self, config::ConfigError> {
        Config::builder()
            .add_source(config::File::with_name(location))
            .add_source(
                Environment::with_prefix(env_prefix)
                    .separator(SETTINGS_SEPARATOR)
                    .prefix_separator(SETTINGS_SEPARATOR),
            )
            .build()?
            .try_deserialize()
    }
}
