use anyhow::{Context, Result};
use serde::Deserialize;

const OPENAI_MODELS: &str = include_str!("../../models/openai.toml");
const ANTHROPIC_MODELS: &str = include_str!("../../models/anthropic.toml");

#[derive(Debug)]
pub struct Models {
    pub openai: Vec<crate::api::openai::Model>,
    pub anthropic: Vec<crate::api::anthropic::Model>,
}

#[derive(Deserialize)]
struct OpenAi {
    pub models: Vec<crate::api::openai::Model>,
}

#[derive(Deserialize)]
struct Anthropic {
    pub models: Vec<crate::api::anthropic::Model>,
}

pub fn get_models_from_template() -> Result<Models> {
    let openai: OpenAi = toml::from_str(OPENAI_MODELS)
        .context("parse openai models toml")?;
    let anthropic: Anthropic = toml::from_str(ANTHROPIC_MODELS)
        .context("parse anthropic models toml")?;
    Ok(Models { openai: openai.models, anthropic: anthropic.models })
}
