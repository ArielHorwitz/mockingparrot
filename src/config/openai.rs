use serde::Deserialize;

use super::ValueRange;

#[derive(Debug, Deserialize, Clone)]
pub struct OpenAi {
    pub key: String,
    pub chat: Chat,
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
    pub temperature: ValueRange<f32>,
    pub top_p: ValueRange<f32>,
    pub frequency_penalty: ValueRange<f32>,
    pub presence_penalty: ValueRange<f32>,
    pub max_tokens: ValueRange<u16>,
}
