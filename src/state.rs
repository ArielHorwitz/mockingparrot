use crate::{api::GptMessage, config::Config};
use anyhow::{Context, Result};
use ratatui::{prelude::Style, style::Color};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tui_textarea::TextArea;

pub mod focus;

pub use focus::Focus;

pub struct State {
    pub config: Config,
    pub hotkey_map: crate::hotkeys::HotkeyMap,
    pub paths: Paths,
    pub conversation: Conversation,
    pub ui: Ui,
}

impl State {
    pub fn new() -> Result<Self> {
        let paths = Paths::generate_dirs().context("generate directories")?;
        let config = crate::config::get_config_from_file(&paths.config_file)
            .context("get config from file")?;
        let system_instructions = config
            .system
            .instructions
            .first()
            .context("no system instructions")?
            .message
            .clone();

        let mut prompt_textarea = TextArea::default();
        prompt_textarea.set_style(
            Style::new()
                .bg(config.ui.colors.prompt.background)
                .fg(config.ui.colors.prompt.foreground),
        );
        prompt_textarea.set_line_number_style(Style::new().bg(Color::Black).fg(Color::Cyan));
        prompt_textarea.set_cursor_style(Style::new().bg(Color::Rgb(200, 200, 200)));
        prompt_textarea.set_cursor_line_style(Style::new());
        let hotkey_map = crate::hotkeys::config_to_map(config.hotkeys.clone());
        let ui = Ui {
            focus: Focus::default(),
            status_bar_text: format!("Welcome to {}", crate::APP_TITLE),
            prompt_textarea,
            conversation_scroll: 0,
            debug_logs: Vec::new(),
            debug_logs_scroll: 0,
            system_instruction_selection: 0,
        };
        let mut state = Self {
            config,
            hotkey_map,
            paths,
            conversation: Conversation::new(system_instructions),
            ui,
        };
        state.add_debug_log("Start of debug logs");
        Ok(state)
    }

    pub fn set_status_bar_text<T: Into<String>>(&mut self, text: T) {
        self.ui.status_bar_text = text.into();
    }

    pub fn add_debug_log<T: Into<String>>(&mut self, log: T) {
        self.ui
            .debug_logs
            .push(format!("{} | {}", get_timestamp(), log.into()));
    }
}

pub struct Paths {
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
    pub config_file: PathBuf,
    pub message_file: PathBuf,
}

impl Paths {
    fn generate_dirs() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .context("get config directory")?
            .join(crate::APP_TITLE.to_lowercase());
        let data_dir = dirs::data_dir()
            .context("get data directory")?
            .join(crate::APP_TITLE.to_lowercase());
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir).context("create config directory")?;
        }
        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir).context("create data directory")?;
        }
        let config_file = config_dir.join("config.toml");
        let message_file = data_dir.join("message_text");
        Ok(Self {
            data_dir,
            config_dir,
            config_file,
            message_file,
        })
    }
}

pub struct Ui {
    pub focus: Focus,
    pub status_bar_text: String,
    pub prompt_textarea: TextArea<'static>,
    pub conversation_scroll: u16,
    pub debug_logs: Vec<String>,
    pub debug_logs_scroll: u16,
    pub system_instruction_selection: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub messages: Vec<GptMessage>,
}

impl Conversation {
    #[must_use]
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
        for message in &self.messages {
            writeln!(f, "{message}")?;
        }
        std::fmt::Result::Ok(())
    }
}

fn get_timestamp() -> String {
    format!("{}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"))
}
