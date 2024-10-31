use crate::app::state::State;
use crate::chat::{Conversation, Message};
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
    state: &State,
    conversation: &Conversation,
) -> Result<CompletionResponse> {
    match state.config.provider {
        Provider::OpenAi => {
            let model = state
                .models
                .openai
                .first()
                .context("no models configured for OpenAI")?;
            openai::get_completion(&state.config.keys.openai, model, conversation)
                .await
                .context("get openai completion")
        }
        Provider::Anthropic => {
            let model = state
                .models
                .anthropic
                .first()
                .context("no models configured for Anthropic")?;
            anthropic::get_completion(&state.config.keys.anthropic, model, conversation)
                .await
                .context("get anthropic completion")
        }
    }
}
