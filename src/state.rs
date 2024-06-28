use crate::{api::GptMessage, config::Config};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct State {
    pub config: Config,
    pub conversation: Conversation,
}

impl State {
    pub fn from_config(config: Config) -> Result<Self> {
        let system_instructions = config
            .system
            .instructions
            .first()
            .context("no system instructions")?
            .message
            .clone();
        Ok(Self {
            config,
            conversation: Conversation::new(system_instructions),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub messages: Vec<GptMessage>,
}

impl Conversation {
    pub fn new(system_instructions: String) -> Self {
        Self {
            messages: vec![GptMessage::new_system_message(system_instructions)],
        }
    }

    pub fn add_message(&mut self, message: GptMessage) {
        self.messages.push(message);
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
