use crate::api::{CompletionResponse, TokenUsage};
use crate::chat::{Conversation, Message as GenericMessage, Role as GenericRole};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const API_ENDPOINT: &str = "https://api.openai.com/v1/chat/completions";

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelClass {
    #[default]
    Classic,
    Reasoning,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub key: String,
    pub models: Vec<Model>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub class: Option<ModelClass>,
    pub max_completion_tokens: u32,
    pub temperature: f32,
    pub top_p: f32,
    pub frequency_penalty: f32,
    pub presence_penalty: f32,
}

impl std::fmt::Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [by OpenAI]", self.name)
    }
}

#[derive(Serialize, Debug)]
struct Request {
    messages: Vec<Message>,
    model: String,
    max_completion_tokens: u32,
    top_p: f32,
    temperature: f32,
    frequency_penalty: f32,
    presence_penalty: f32,
}

impl Request {
    fn new(model: &Model, conversation: &Conversation) -> Self {
        let instruction_role = match model.class.unwrap_or_default() {
            ModelClass::Classic => Role::System,
            // we use the 'user' role because it seems some o1 models do not
            // support developer/system instructions yet
            ModelClass::Reasoning => Role::User,
        };
        let system_message = Message {
            role: instruction_role,
            content: conversation.system_instructions.clone(),
        };
        let mut messages = vec![system_message];
        messages.extend(&mut conversation.messages.iter().map(std::convert::Into::into));
        Self {
            messages,
            model: model.id.clone(),
            max_completion_tokens: model.max_completion_tokens,
            temperature: model.temperature,
            top_p: model.top_p,
            frequency_penalty: model.frequency_penalty,
            presence_penalty: model.presence_penalty,
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

impl From<&GenericRole> for Role {
    fn from(value: &GenericRole) -> Self {
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
            role: (&value.role).into(),
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
    key: &str,
    model: &Model,
    conversation: &Conversation,
) -> Result<CompletionResponse> {
    let client = reqwest::Client::new();
    let call_data = Request::new(model, conversation);
    let raw_response = client
        .post(API_ENDPOINT)
        .bearer_auth(key)
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
        role: GenericRole::Assistant(model.to_string()),
        content: message.content.clone(),
    };
    let response = CompletionResponse {
        message: generic_message,
        usage: parsed_response.usage.into(),
    };
    Ok(response)
}
