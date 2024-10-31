use crate::api::Provider;
use crate::app::hotkeys::HotkeyConfig;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

mod system;
mod ui;
mod value_range;

pub use value_range::ValueRange;

const CONFIG_TEMPLATE: &str = include_str!("../config.template.toml");
const OPENAI_MODELS_TEMPLATE: &str = include_str!("../models/openai.toml");
const ANTHROPIC_MODELS_TEMPLATE: &str = include_str!("../models/anthropic.toml");

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

#[derive(Debug)]
pub struct Models {
    pub openai: Vec<crate::api::openai::Model>,
    pub anthropic: Vec<crate::api::anthropic::Model>,
}

impl Models {
    pub fn from_disk(models_dir: &Path, generate_missing: bool) -> Result<Self> {
        let openai_models_file = models_dir.join(format!("{}.toml", Provider::OpenAi));
        let anthropic_models_file = models_dir.join(format!("{}.toml", Provider::Anthropic));
        let openai = get_openai_models_from_file(&openai_models_file, generate_missing)?;
        let anthropic = get_anthropic_models_from_file(&anthropic_models_file, generate_missing)?;
        Ok(Self { openai, anthropic })
    }
}

#[derive(Deserialize)]
struct OpenAi {
    pub models: Vec<crate::api::openai::Model>,
}

#[derive(Deserialize)]
struct Anthropic {
    pub models: Vec<crate::api::anthropic::Model>,
}

fn get_openai_models_from_file(
    config_file: &Path,
    generate_missing: bool,
) -> Result<Vec<crate::api::openai::Model>> {
    // Create config from template if missing
    if !config_file.exists() && generate_missing {
        std::fs::write(config_file, OPENAI_MODELS_TEMPLATE)
            .context("generate missing openai models file from template")?;
    }
    let models_file_contents =
        std::fs::read_to_string(config_file).context("read openai models file")?;
    Ok(toml::from_str::<OpenAi>(&models_file_contents)
        .context("parse openai models toml")?
        .models)
}

fn get_anthropic_models_from_file(
    config_file: &Path,
    generate_missing: bool,
) -> Result<Vec<crate::api::anthropic::Model>> {
    // Create config from template if missing
    if !config_file.exists() && generate_missing {
        std::fs::write(config_file, ANTHROPIC_MODELS_TEMPLATE)
            .context("generate missing anthropic models file from template")?;
    }
    let models_file_contents =
        std::fs::read_to_string(config_file).context("read anthropic models file")?;
    Ok(toml::from_str::<Anthropic>(&models_file_contents)
        .context("parse anthropic models toml")?
        .models)
}

#[cfg(test)]
mod config_tests {
    use super::{Config, Models};
    use std::path::Path;

    #[test]
    fn config_template() {
        let template_file = Path::new("config.template.toml");
        Config::from_file(template_file, false).expect("load config from template");
    }

    #[test]
    fn models_templates() {
        let models_dir = Path::new("models");
        Models::from_disk(models_dir, false).expect("load openai models from template");
    }
}
