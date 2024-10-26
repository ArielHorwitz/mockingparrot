use crate::chat::{Conversation, Message};
use crate::config::Config;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub mod anthropic;
pub mod openai;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Provider {
    #[serde(rename = "openai")]
    OpenAi,
    #[serde(rename = "anthropic")]
    Anthropic,
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let serialized_name = serde_json::to_string(self).map_err(|_| std::fmt::Error)?;
        write!(f, "{}", serialized_name.trim_matches('"'))
    }
}

pub struct CompletionResponse {
    pub message: Message,
    pub usage: TokenUsage,
}

pub struct TokenUsage {
    pub prompt: u32,
    pub completion: u32,
    pub total: u32,
}

impl std::fmt::Display for TokenUsage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Tokens: {} [{} prompt, {} completion]",
            self.total, self.prompt, self.completion
        )
    }
}

pub async fn get_completion(
    config: &Config,
    conversation: &Conversation,
) -> Result<CompletionResponse> {
    match config.provider {
        Provider::OpenAi => openai::get_completion(&config.openai, conversation)
            .await
            .context("get openai completion"),
        Provider::Anthropic => anthropic::get_completion(&config.anthropic, conversation)
            .await
            .context("get anthropic completion"),
    }
}
