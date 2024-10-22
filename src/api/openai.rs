use crate::api::{CompletionResponse, Provider, TokenUsage};
use crate::config::openai::Chat as ChatConfig;
use crate::config::Config;
use crate::conversation::{Conversation, Message, Role};
use anyhow::{Context, Result};
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
            max_tokens: config.max_tokens.value.try_into().expect("max tokens"),
            temperature: config.temperature.value,
            top_p: config.top_p.value,
            frequency_penalty: config.frequency_penalty.value,
            presence_penalty: config.presence_penalty.value,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum GptRole {
    User,
    System,
    Assistant,
}

impl From<GptRole> for Role {
    fn from(value: GptRole) -> Self {
        match value {
            GptRole::System => Self::System,
            GptRole::Assistant => Self::Assistant(Provider::OpenAi),
            GptRole::User => Self::User,
        }
    }
}

impl From<Role> for GptRole {
    fn from(value: Role) -> Self {
        match value {
            Role::System => Self::System,
            Role::Assistant(_) => Self::Assistant,
            Role::User => Self::User,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GptMessage {
    pub role: GptRole,
    pub content: String,
}

impl GptMessage {
    #[must_use]
    pub fn new_user_message(content: String) -> Self {
        GptMessage {
            role: GptRole::User,
            content,
        }
    }

    #[must_use]
    pub fn new_system_message(content: String) -> Self {
        GptMessage {
            role: GptRole::System,
            content,
        }
    }
}

impl From<&Message> for GptMessage {
    fn from(value: &Message) -> Self {
        Self {
            role: value.role.into(),
            content: value.content.clone(),
        }
    }
}

impl From<&GptMessage> for Message {
    fn from(value: &GptMessage) -> Self {
        Self {
            role: value.role.into(),
            content: value.content.clone(),
        }
    }
}

impl std::fmt::Display for GptMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.role, self.content)
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
#[allow(clippy::struct_field_names)]
pub struct GptResponseUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl From<GptResponseUsage> for TokenUsage {
    fn from(value: GptResponseUsage) -> Self {
        TokenUsage {
            prompt: value.prompt_tokens,
            completion: value.completion_tokens,
            total: value.total_tokens,
        }
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

#[derive(Deserialize, Debug)]
pub struct GptErrorContainer {
    pub error: GptError,
}

#[derive(Deserialize, Debug)]
#[allow(unused)]
pub struct GptError {
    pub message: String,
    pub r#type: String,
    pub param: String,
    pub code: Option<String>,
}

pub async fn get_completion(
    config: &Config,
    conversation: &Conversation,
) -> Result<CompletionResponse> {
    let client = reqwest::Client::new();
    let call_data = GptRequest::new(
        &config.openai.chat,
        conversation
            .messages
            .iter()
            .map(std::convert::Into::into)
            .collect(),
    );
    let raw_response = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(&config.openai.key)
        .json(&call_data)
        .send()
        .await
        .context("send api request")?
        .text()
        .await
        .context("parse api response as json")?;
    match serde_json::from_str::<GptResponse>(&raw_response) {
        Ok(gpt_response) => {
            let message = &gpt_response
                .choices
                .first()
                .context("missing response choices")?
                .message;
            let response = CompletionResponse {
                message: message.into(),
                usage: gpt_response.usage.into(),
            };
            Ok(response)
        }
        Err(response_parse_error) => match serde_json::from_str::<GptErrorContainer>(&raw_response)
        {
            Ok(error) => Err(anyhow::Error::msg(error.error.message.to_string())),
            Err(_error_parse_error) => {
                let error_message =
                    format!("failed to parse response: {response_parse_error}\n{raw_response}");
                Err(anyhow::Error::msg(error_message))
            }
        },
    }
}
