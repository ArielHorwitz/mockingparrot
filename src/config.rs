use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

pub const CONFIG_FILE_PATH: &str = ".config/hummingparrot/config.toml";
const CONFIG_TEMPLATE: &str = include_str!("../config.template.toml");

use crate::hotkeys::{get_default_config, HotkeyConfig};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub api: Api,
    pub chat: Chat,
    pub ui: Ui,
    pub system: System,
    #[serde(default = "get_default_config")]
    pub hotkeys: HotkeyConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Api {
    pub key: String,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize, Clone, Copy)]
pub enum ChatModel {
    #[serde(rename = "gpt-4o")]
    Gpt_4o,
}

impl std::fmt::Display for ChatModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            ChatModel::Gpt_4o => "gpt-4o",
        };
        write!(f, "{name}")
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Chat {
    pub model: ChatModel,
    pub max_tokens: i16,
    pub temperature: f32,
    pub top_p: f32,
    pub frequency_penalty: f32,
    pub presence_penalty: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Ui {
    pub editor_command: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct System {
    pub instructions: Vec<SystemInstructions>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SystemInstructions {
    pub name: String,
    pub message: String,
}

pub fn get_config_from_file() -> Result<Config> {
    // Resolve config file path
    let user_home_dir = std::env::var("HOME").context("get HOME environment variable")?;
    let config_file = Path::new(&user_home_dir).join(CONFIG_FILE_PATH);
    // Create config directory if missing
    let config_dir = config_file.parent().context("get config directory")?;
    if !config_dir.exists() {
        std::fs::create_dir_all(config_dir).context("create config directory")?;
    }
    // Create config from template if missing
    if !config_file.exists() {
        std::fs::write(&config_file, CONFIG_TEMPLATE).context("write config template to file")?;
    }
    // Read and parse file
    let config_file_contents = std::fs::read_to_string(config_file).context("read config file")?;
    toml::from_str(&config_file_contents).context("parse config file toml")
}
