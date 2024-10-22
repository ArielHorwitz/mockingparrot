use crate::hotkeys::HotkeyConfig;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::ops::{Add, Sub};

pub mod openai;
mod ui;

const CONFIG_TEMPLATE: &str = include_str!("../config.template.toml");

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub provider: crate::api::Provider,
    pub openai: openai::OpenAi,
    pub ui: ui::Ui,
    pub commands: Commands,
    pub system: System,
    pub hotkeys: HotkeyConfig,
}

#[derive(Debug, Deserialize, Copy, Clone)]
pub struct ValueRange<T> {
    pub value: T,
    pub min: T,
    pub max: T,
    #[serde(alias = "step")]
    pub increment_step: T,
}

impl<T> ValueRange<T>
where
    T: Copy + Add<Output = T> + Sub<Output = T> + PartialOrd,
{
    pub fn increment(&mut self) {
        self.value = num_traits::clamp(self.value.add(self.increment_step), self.min, self.max);
    }

    pub fn decrement(&mut self) {
        self.value = num_traits::clamp(self.value.sub(self.increment_step), self.min, self.max);
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Commands {
    pub editor: Vec<String>,
    pub copy: Vec<String>,
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

impl SystemInstructions {
    #[must_use]
    pub fn preview(&self, length: usize) -> String {
        self.message
            .chars()
            .take(length)
            .map(|char| if char == '\n' { ' ' } else { char })
            .collect()
    }
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
