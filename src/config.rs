use anyhow::{Context, Result};
use ratatui::prelude::Color;
use serde::Deserialize;
use std::ops::{Add, Sub};

const CONFIG_TEMPLATE: &str = include_str!("../config.template.toml");

use crate::hotkeys::HotkeyConfig;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub api: Api,
    pub chat: Chat,
    pub ui: Ui,
    pub commands: Commands,
    pub system: System,
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
    pub max_tokens: ValueRange<u16>,
    pub temperature: ValueRange<f32>,
    pub top_p: ValueRange<f32>,
    pub frequency_penalty: ValueRange<f32>,
    pub presence_penalty: ValueRange<f32>,
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
pub struct Ui {
    pub layout: Layout,
    pub colors: Colors,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Layout {
    pub prompt_size: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Colors {
    pub text: ColorVariants,
    pub background: ColorVariants,
    pub frame: ColorVariants,
    pub widget: ColorVariants,
    pub cursor: ColorVariants,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ColorVariants {
    pub normal: Color,
    pub inactive: Color,
    pub highlight: Color,
    pub title: Color,
    pub warn: Color,
}

impl ColorVariants {
    #[must_use]
    pub fn get_active(&self, active: bool) -> Color {
        if active {
            self.normal
        } else {
            self.inactive
        }
    }

    #[must_use]
    pub fn get_highlight(&self, highlight: bool) -> Color {
        if highlight {
            self.highlight
        } else {
            self.normal
        }
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
