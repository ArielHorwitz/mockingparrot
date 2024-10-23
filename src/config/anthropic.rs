use serde::{Deserialize, Serialize};

use super::ValueRange;

#[derive(Debug, Deserialize, Clone)]
pub struct Anthropic {
    pub key: String,
    pub chat: Chat,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum ChatModel {
    #[serde(rename = "claude-3-5-sonnet")]
    Claude_3_5_Sonnet,
    #[serde(rename = "claude-3-sonnet")]
    Claude_3_Sonnet,
    #[serde(rename = "claude-3-opus")]
    Claude_3_Opus,
    #[serde(rename = "claude-3-haiku")]
    Claude_3_Haiku,
}

impl std::fmt::Display for ChatModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let serialized_name = serde_json::to_string(self).map_err(|_| std::fmt::Error)?;
        write!(f, "{}-latest", serialized_name.trim_matches('"'))
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Chat {
    pub model: ChatModel,
    pub temperature: ValueRange<f32>,
    pub max_tokens: ValueRange<u16>,
}
