use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub api: Api,
    pub chat: ChatConfig,
    pub ui: Ui,
    pub system: System,
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
