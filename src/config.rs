use crate::app::hotkeys::HotkeyConfig;
use anyhow::{Context, Result};
use serde::Deserialize;

mod system;
mod ui;
mod value_range;

pub use value_range::ValueRange;

const CONFIG_TEMPLATE: &str = include_str!("../config.template.toml");

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub model: crate::api::ProviderModel,
    pub openai: crate::api::openai::Config,
    pub anthropic: crate::api::anthropic::Config,
    pub ui: ui::Ui,
    pub commands: Commands,
    pub system: system::System,
    pub hotkeys: HotkeyConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Commands {
    pub editor: Vec<String>,
    pub copy: Vec<String>,
}

pub fn get_config_from_file(config_file: &std::path::Path) -> Result<Config> {
    // Create config from template if missing
    if !config_file.exists() {
        std::fs::write(config_file, CONFIG_TEMPLATE).context("write config template to file")?;
    }
    // Read and parse file
    let config_file_contents = std::fs::read_to_string(config_file).context("read config file")?;
    toml::from_str(&config_file_contents).context("parse config file toml")
}
