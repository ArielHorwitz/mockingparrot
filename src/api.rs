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
    match state.ui.selected_model {
        crate::app::state::SelectedModel::OpenAi(model_index) => {
            let model = state
                .models
                .openai
                .get(model_index)
                .context("model index out of range")?;
            openai::get_completion(&state.config.keys.openai, model, conversation)
                .await
                .context("get openai completion")
        }
        crate::app::state::SelectedModel::Anthropic(model_index) => {
            let model = state
                .models
                .anthropic
                .get(model_index)
                .context("model index out of range")?;
            anthropic::get_completion(&state.config.keys.anthropic, model, conversation)
                .await
                .context("get anthropic completion")
        }
    }
}
