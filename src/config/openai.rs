use serde::Deserialize;

use super::ValueRange;

#[derive(Debug, Deserialize, Clone)]
pub struct OpenAi {
    pub key: String,
    pub chat: Chat,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Chat {
    pub temperature: ValueRange<f32>,
    pub top_p: ValueRange<f32>,
    pub frequency_penalty: ValueRange<f32>,
    pub presence_penalty: ValueRange<f32>,
    pub max_tokens: ValueRange<u16>,
}
