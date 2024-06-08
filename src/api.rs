use crate::config::{ChatConfig, Config};
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const SYSTEM_MESSAGE: &str = "You are a helpful assistant. Do not bother being polite. Your responses should be concise and terse.";

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

#[derive(Deserialize, Serialize, Debug)]
pub struct GptMessage {
    pub role: String,
    pub content: String,
}

impl GptMessage {
    fn new_user_message(content: String) -> Self {
        GptMessage {
            role: "user".to_owned(),
            content,
        }
    }

    fn new_system_message(content: String) -> Self {
        GptMessage {
            role: "system".to_owned(),
            content,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct GptResponseChoice {
    pub index: u16,
    pub message: GptMessage,
    pub logprobs: Option<String>,
    pub finish_reason: String,
}

#[derive(Deserialize, Debug)]
pub struct GptResponseUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

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

pub async fn call_api(client: &Client, config: &Config, message: &str) -> Result<GptResponse> {
    let call_data = GptRequest::new(
        &config.chat,
        vec![
            GptMessage::new_system_message(SYSTEM_MESSAGE.to_owned()),
            GptMessage::new_user_message(message.to_owned()),
        ],
    );
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
