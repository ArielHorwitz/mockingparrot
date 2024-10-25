use crate::api::{CompletionResponse, TokenUsage};
use crate::chat::{Conversation, Message as GenericMessage, Role as GenericRole};
use crate::config::ValueRange;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const MESSAGES_URL: &str = "https://api.anthropic.com/v1/messages";
const MODEL_VERSION: &str = "2023-06-01";

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub key: String,
    pub chat: ChatConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChatConfig {
    pub temperature: ValueRange<f32>,
    pub max_tokens: ValueRange<u16>,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum Model {
    #[serde(rename = "claude-3-5-sonnet")]
    Claude_3_5_Sonnet,
    #[serde(rename = "claude-3-sonnet")]
    Claude_3_Sonnet,
    #[serde(rename = "claude-3-opus")]
    Claude_3_Opus,
    #[serde(rename = "claude-3-haiku")]
    Claude_3_Haiku,
}

impl std::fmt::Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let serialized_name = serde_json::to_string(self).map_err(|_| std::fmt::Error)?;
        write!(f, "{}-latest", serialized_name.trim_matches('"'))
    }
}

#[derive(Serialize, Debug)]
struct Request {
    messages: Vec<Message>,
    model: String,
    max_tokens: i16,
    temperature: f32,
    system: String,
}

impl Request {
    fn new(model: Model, config: &ChatConfig, conversation: &Conversation) -> Self {
        let messages = conversation
            .messages
            .iter()
            .map(std::convert::Into::into)
            .collect();
        Request {
            messages,
            model: model.to_string(),
            max_tokens: config.max_tokens.value.try_into().expect("max tokens"),
            temperature: config.temperature.value,
            system: conversation.system_instructions.clone(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Role {
    User,
    Assistant,
}

impl From<GenericRole> for Role {
    fn from(value: GenericRole) -> Self {
        match value {
            GenericRole::Assistant(_) => Self::Assistant,
            GenericRole::User => Self::User,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Message {
    pub role: Role,
    pub content: String,
}

impl From<&GenericMessage> for Message {
    fn from(value: &GenericMessage) -> Self {
        Self {
            role: value.role.into(),
            content: value.content.clone(),
        }
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.role, self.content)
    }
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
struct ResponseContent {
    pub r#type: String,
    pub text: String,
}

#[derive(Deserialize, Debug)]
struct ResponseUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

impl From<ResponseUsage> for TokenUsage {
    fn from(value: ResponseUsage) -> Self {
        TokenUsage {
            prompt: value.input_tokens,
            completion: value.output_tokens,
            total: value.input_tokens + value.output_tokens,
        }
    }
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
struct Response {
    pub id: String,
    pub r#type: String,
    pub role: Role,
    pub model: String,
    pub content: Vec<ResponseContent>,
    pub stop_reason: String,
    pub stop_sequence: Option<String>,
    pub usage: ResponseUsage,
}

pub async fn get_completion(
    model: Model,
    config: &Config,
    conversation: &Conversation,
) -> Result<CompletionResponse> {
    let client = reqwest::Client::new();
    let call_data = Request::new(model, &config.chat, conversation);
    let raw_response = client
        .post(MESSAGES_URL)
        .header("x-api-key", &config.key)
        .header("anthropic-version", MODEL_VERSION)
        .header("content-type", "application/json")
        .json(&call_data)
        .send()
        .await
        .context("send api request")?
        .text()
        .await
        .context("parse api response as json")?;
    let parsed_response = serde_json::from_str::<Response>(&raw_response)
        .with_context(|| format!("failed to parse response: {raw_response}"))?;
    if parsed_response.role != Role::Assistant {
        anyhow::bail!("unexpected non-assistant role response");
    };
    let message_content = &parsed_response
        .content
        .first()
        .context("missing response choices")?
        .text;
    let message = GenericMessage {
        role: GenericRole::Assistant(crate::api::ProviderModel::Anthropic(model)),
        content: message_content.clone(),
    };
    let response = CompletionResponse {
        message,
        usage: parsed_response.usage.into(),
    };
    Ok(response)
}
