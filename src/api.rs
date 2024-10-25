use crate::chat::{Conversation, Message};
use crate::config::Config;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub mod anthropic;
pub mod openai;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "provider", content = "model")]
pub enum ProviderModel {
    OpenAi(openai::Model),
    Anthropic(anthropic::Model),
}

impl std::fmt::Display for ProviderModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenAi(model) => write!(f, "OpenAI [{model}]"),
            Self::Anthropic(model) => write!(f, "Anthropic [{model}]"),
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
    match config.model {
        ProviderModel::OpenAi(model) => openai::get_completion(model, &config.openai, conversation)
            .await
            .context("get openai completion"),
        ProviderModel::Anthropic(model) => {
            anthropic::get_completion(model, &config.anthropic, conversation)
                .await
                .context("get anthropic completion")
        }
    }
}
