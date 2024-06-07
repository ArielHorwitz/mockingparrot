// For details see: https://platform.openai.com/docs/api-reference/chat

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub api: Api,
    pub chat: ChatConfig,
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
pub struct ChatConfig {
    pub model: ChatModel,
    pub max_tokens: i16,
    pub temperature: f32,
    pub top_p: f32,
    pub frequency_penalty: f32,
    pub presence_penalty: f32,
}

impl Default for ChatConfig {
    fn default() -> Self {
        Self {
            model: ChatModel::Gpt_4o,
            max_tokens: 4096,
            temperature: 1.0,
            top_p: 1.0,
            frequency_penalty: 0.05,
            presence_penalty: 0.01,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        let api = Api {
            key: "Enter you api key here".to_owned(),
        };
        Self {
            api,
            chat: ChatConfig::default(),
        }
    }
}
