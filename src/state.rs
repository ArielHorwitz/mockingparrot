use crate::{api::GptMessage, config::Config};
use serde::{Deserialize, Serialize};

const DEFAULT_SYSTEM_MESSAGE: &str = "You are a helpful assistant. Do not bother being polite. Your responses should be concise and terse.";

#[derive(Debug)]
pub struct State {
    pub config: Config,
    pub conversation: Conversation,
    pub status_bar_text: String,
}

impl State {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            conversation: Conversation::default(),
            status_bar_text: format!("Welcome to {}", crate::APP_TITLE),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub messages: Vec<GptMessage>,
}

impl Conversation {
    pub fn add_message(&mut self, message: GptMessage) {
        self.messages.push(message);
    }
}

impl Default for Conversation {
    fn default() -> Self {
        Self {
            messages: vec![GptMessage::new_system_message(
                DEFAULT_SYSTEM_MESSAGE.to_owned(),
            )],
        }
    }
}

impl std::fmt::Display for Conversation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for message in self.messages.iter() {
            writeln!(f, "{message}")?;
        }
        std::fmt::Result::Ok(())
    }
}
