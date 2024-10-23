use crate::config::Config;
use crate::conversation::{Conversation, Message};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

mod anthropic;
mod openai;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    OpenAi,
    Anthropic,
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenAi => write!(f, "OpenAI"),
            Self::Anthropic => write!(f, "Anthropic"),
        }
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
        Provider::OpenAi => openai::get_completion(config, conversation)
            .await
            .context("get openai completion"),
        Provider::Anthropic => anthropic::get_completion(config, conversation)
            .await
            .context("get anthropic completion"),
    }
}
