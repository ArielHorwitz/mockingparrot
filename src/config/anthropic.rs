use serde::Deserialize;

use super::ValueRange;

#[derive(Debug, Deserialize, Clone)]
pub struct Anthropic {
    pub key: String,
    pub chat: Chat,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Chat {
    pub temperature: ValueRange<f32>,
    pub max_tokens: ValueRange<u16>,
}
