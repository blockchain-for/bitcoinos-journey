use config::{Config, File, Environment};
use serde::{Deserialize, Serialize};

const SETTINGS_SEPARATOR: &str = "__";

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Settings {
    pub descriptor: Option<String>,
}

impl Settings {
    pub fn load(location: &str, env_prefix: &str) -> anyhow::Result<Self> {
        let settings = Config::builder()
            .add_source(File::with_name(location))
            .add_source(
                Environment::with_prefix(env_prefix)
                    .separator(SETTINGS_SEPARATOR)
                    .prefix_separator(SETTINGS_SEPARATOR)
            )
            .build()?;
        let settings = settings.try_deserialize()?;

        Ok(settings)
    }
}
