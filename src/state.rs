use crate::{api::GptMessage, config::Config};
use anyhow::{Context, Result};
use ratatui::{prelude::Style, style::Color, widgets::ListState};
use serde::{Deserialize, Serialize};
use tui_textarea::TextArea;

pub struct State {
    pub config: Config,
    pub conversation: Conversation,
    pub tab: ViewTab,
    pub status_bar_text: String,
    pub prompt_textarea: TextArea<'static>,
    pub debug_logs: Vec<String>,
    pub system_instruction_selection: ListState,
    pub debug: bool,
    pub key_event_debug: String,
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

        let mut prompt_textarea = TextArea::default();
        prompt_textarea.set_style(Style::new().bg(Color::Rgb(0, 25, 25)).fg(Color::White));
        prompt_textarea.set_line_number_style(Style::new().bg(Color::Black).fg(Color::Cyan));
        prompt_textarea.set_cursor_style(Style::new().bg(Color::Rgb(200, 200, 200)));
        Ok(Self {
            config,
            conversation: Conversation::new(system_instructions),
            tab: ViewTab::Conversation,
            status_bar_text: format!("Welcome to {}", crate::APP_TITLE),
            prompt_textarea,
            debug_logs: vec!["Debug logs empty.".to_string()],
            system_instruction_selection: ListState::default().with_selected(Some(0)),
            debug: false,
            key_event_debug: Default::default(),
        })
    }

    pub fn set_status_bar_text<T: Into<String>>(&mut self, text: T) {
        self.status_bar_text = text.into();
    }

    pub fn add_debug_log<T: Into<String>>(&mut self, log: T) {
        self.debug_logs.push(log.into());
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ViewTab {
    Conversation,
    NewConversation,
    Config,
}

impl ViewTab {
    pub fn next_tab(self) -> ViewTab {
        match self {
            ViewTab::Conversation => ViewTab::NewConversation,
            ViewTab::NewConversation => ViewTab::Config,
            ViewTab::Config => ViewTab::Conversation,
        }
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
