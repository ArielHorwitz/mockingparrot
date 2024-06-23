use crate::config::{ChatConfig, Config};
use crate::state::Conversation;
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
struct GptRequest {
    messages: Vec<GptMessage>,
    model: String,
    top_p: f32,
    max_tokens: i16,
    temperature: f32,
    frequency_penalty: f32,
    presence_penalty: f32,
}

impl GptRequest {
    fn new(config: &ChatConfig, messages: Vec<GptMessage>) -> Self {
        GptRequest {
            messages,
            model: config.model.to_string(),
            max_tokens: config.max_tokens,
            temperature: config.temperature,
            top_p: config.top_p,
            frequency_penalty: config.frequency_penalty,
            presence_penalty: config.presence_penalty,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GptMessage {
    pub role: String,
    pub content: String,
}

impl GptMessage {
    pub fn new_user_message(content: String) -> Self {
        GptMessage {
            role: "user".to_owned(),
            content,
        }
    }

    pub fn new_system_message(content: String) -> Self {
        GptMessage {
            role: "system".to_owned(),
            content,
        }
    }
}

impl std::fmt::Display for GptMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.role, self.content)
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct GptResponseChoice {
    pub index: u16,
    pub message: GptMessage,
    pub logprobs: Option<()>,
    pub finish_reason: String,
}

#[derive(Deserialize, Debug)]
pub struct GptResponseUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl std::fmt::Display for GptResponseUsage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Tokens: {} [{} prompt, {} completion]",
            self.total_tokens, self.prompt_tokens, self.completion_tokens
        )
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct GptResponse {
    pub id: String,
    pub object: String,
    pub created: u128,
    pub model: String,
    pub choices: Vec<GptResponseChoice>,
    pub usage: GptResponseUsage,
    pub system_fingerprint: String,
}

pub async fn call_api(
    client: &Client,
    config: &Config,
    conversation: &Conversation,
) -> Result<GptResponse> {
    let call_data = GptRequest::new(&config.chat, conversation.messages.clone());
    let response: GptResponse = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(&config.api.key)
        .json(&call_data)
        .send()
        .await?
        .json()
        .await?;
    Ok(response)
}
