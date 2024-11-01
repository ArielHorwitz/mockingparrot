use crate::api::Provider;
use crate::app::hotkeys::HotkeyConfig;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

mod models;
mod system;
mod ui;
mod value_range;

pub use models::Models;
pub use value_range::ValueRange;

const CONFIG_TEMPLATE: &str = include_str!("../config.template.toml");

#[derive(Deserialize)]
pub struct Config {
    pub provider: Provider,
    pub keys: ApiKeys,
    pub ui: ui::Ui,
    pub commands: Commands,
    pub system: system::System,
    pub hotkeys: HotkeyConfig,
}

impl Config {
    pub fn from_file(config_file: &Path, generate_missing: bool) -> Result<Self> {
        // Create config from template if missing
        if !config_file.exists() && generate_missing {
            std::fs::write(config_file, CONFIG_TEMPLATE)
                .context("write config template to file")?;
        }
        // Read and parse file
        let config_file_contents =
            std::fs::read_to_string(config_file).context("read config file")?;
        toml::from_str(&config_file_contents).context("parse config file toml")
    }
}

#[derive(Debug, Deserialize)]
pub struct ApiKeys {
    pub openai: String,
    pub anthropic: String,
}

#[derive(Debug, Deserialize)]
pub struct Commands {
    pub editor: Vec<String>,
    pub copy: Vec<String>,
}

#[cfg(test)]
mod config_tests {
    use super::Config;
    use std::path::Path;

    #[test]
    fn config_template() {
        let template_file = Path::new("config.template.toml");
        Config::from_file(template_file, false).expect("load config from template");
    }
}
