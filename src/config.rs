use crate::app::hotkeys::HotkeyConfig;
use anyhow::{Context, Result};
use serde::Deserialize;

mod models;
mod system;
mod ui;
mod value_range;

pub use value_range::ValueRange;

const CONFIG_TEMPLATE: &str = include_str!("../config.template.toml");

#[derive(Deserialize)]
struct ParsedConfigFile {
    pub provider: crate::api::Provider,
    pub keys: ApiKeys,
    pub ui: ui::Ui,
    pub commands: Commands,
    pub system: system::System,
    pub hotkeys: HotkeyConfig,
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

#[derive(Debug)]
pub struct Config {
    pub provider: crate::api::Provider,
    pub models: models::Models,
    pub keys: ApiKeys,
    pub ui: ui::Ui,
    pub commands: Commands,
    pub system: system::System,
    pub hotkeys: HotkeyConfig,
}

pub fn get_config_from_file(config_file: &std::path::Path) -> Result<Config> {
    // Create config from template if missing
    if !config_file.exists() {
        std::fs::write(config_file, CONFIG_TEMPLATE).context("write config template to file")?;
    }
    // Read and parse file
    let config_file_contents = std::fs::read_to_string(config_file).context("read config file")?;
    let parsed_config_file: ParsedConfigFile = toml::from_str(&config_file_contents).context("parse config file toml")?;
    let config = Config {
        provider: parsed_config_file.provider,
        models: models::get_models_from_template()?,
        keys: parsed_config_file.keys,
        ui: parsed_config_file.ui,
        commands: parsed_config_file.commands,
        system: parsed_config_file.system,
        hotkeys: parsed_config_file.hotkeys,
    };
    Ok(config)
}
