use crate::config::{ChatConfig, Config};
use anyhow::Result;
use reqwest::{Client, Response};
use serde::Serialize;

const SYSTEM_MESSAGE: &str = "You are a helpful assistant. Do not bother being polite. Your responses should be concise and terse.";

#[derive(Serialize)]
struct GptRequest {
    messages: Vec<GptMessage>,
    model: String,
    top_p: f32,
    max_tokens: i16,
    temperature: f32,
    frequency_penalty: f32,
    presence_penalty: f32,
}

impl From<&ChatConfig> for GptRequest {
    fn from(val: &ChatConfig) -> Self {
        GptRequest {
            messages: Vec::new(),
            model: val.model.to_string(),
            max_tokens: val.max_tokens,
            temperature: val.temperature,
            top_p: val.top_p,
            frequency_penalty: val.frequency_penalty,
            presence_penalty: val.presence_penalty,
        }
    }
}

#[derive(Serialize)]
struct GptMessage {
    role: String,
    content: String,
}

pub async fn call_api(client: &Client, config: &Config, message: &str) -> Result<Response> {
    let system_message = GptMessage {
        role: "system".to_owned(),
        content: SYSTEM_MESSAGE.to_owned(),
    };
    let user_message = GptMessage {
        role: "user".to_owned(),
        content: message.to_owned(),
    };
    let mut call_data: GptRequest = (&config.chat).into();
    call_data.messages = Vec::from([system_message, user_message]);
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(&config.api.key)
        .json(&call_data)
        .send()
        .await?;
    Ok(response)
}
