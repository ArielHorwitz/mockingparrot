use crate::api::{CompletionResponse, Provider, TokenUsage};
use crate::config::openai::Chat as ChatConfig;
use crate::config::Config;
use crate::conversation::{Conversation, Message as GenericMessage, Role as GenericRole};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
struct Request {
    messages: Vec<Message>,
    model: String,
    top_p: f32,
    max_tokens: i16,
    temperature: f32,
    frequency_penalty: f32,
    presence_penalty: f32,
}

impl Request {
    fn new(config: &ChatConfig, messages: Vec<Message>) -> Self {
        Self {
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

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Role {
    User,
    System,
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

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct ResponseChoice {
    pub index: u16,
    pub message: Message,
    pub logprobs: Option<()>,
    pub finish_reason: String,
}

#[derive(Deserialize, Debug)]
#[allow(clippy::struct_field_names)]
struct ResponseUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl From<ResponseUsage> for TokenUsage {
    fn from(value: ResponseUsage) -> Self {
        TokenUsage {
            prompt: value.prompt_tokens,
            completion: value.completion_tokens,
            total: value.total_tokens,
        }
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Response {
    pub id: String,
    pub object: String,
    pub created: u128,
    pub model: String,
    pub choices: Vec<ResponseChoice>,
    pub usage: ResponseUsage,
    pub system_fingerprint: String,
}

pub async fn get_completion(
    config: &Config,
    conversation: &Conversation,
) -> Result<CompletionResponse> {
    let client = reqwest::Client::new();
    let system_message = Message {
        role: Role::System,
        content: conversation.system_instructions.clone(),
    };
    let mut messages = vec![system_message];
    messages.extend(&mut conversation.messages.iter().map(std::convert::Into::into));
    let call_data = Request::new(&config.openai.chat, messages);
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
    let parsed_response = serde_json::from_str::<Response>(&raw_response)
        .with_context(|| format!("failed to parse response: {raw_response}"))?;
    let message = &parsed_response
        .choices
        .first()
        .context("missing response choices")?
        .message;
    if message.role != Role::Assistant {
        anyhow::bail!("unexpected non-assistant role response");
    };
    let generic_message = GenericMessage {
        role: GenericRole::Assistant(Provider::OpenAi),
        content: message.content.clone(),
    };
    let response = CompletionResponse {
        message: generic_message,
        usage: parsed_response.usage.into(),
    };
    Ok(response)
}
