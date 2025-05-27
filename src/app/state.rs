use crate::{
    app::hotkeys,
    chat::Conversation,
    config::{get_models_file, Config, Models, Theme},
};
use anyhow::{Context, Result};
use std::io::Write;
use std::path::PathBuf;

mod model;
mod paths;
mod ui;

pub use model::SelectedModel;
pub use paths::Paths;

pub struct State {
    pub config: Config,
    pub models: Models,
    pub theme: Theme,
    pub hotkey_map: hotkeys::HotkeyMap,
    pub paths: Paths,
    pub conversations: Vec<Conversation>,
    pub ui: ui::Ui,
}

impl State {
    pub fn new() -> Result<Self> {
        let paths = Paths::generate_dirs().context("generate directories")?;
        let config =
            Config::from_file(&paths.get_config_file(), true).context("get config from disk")?;
        let models = Models::from_disk(&paths.models_dir, true).context("get models from disk")?;
        Theme::generate_default(&paths.get_theme_file()).context("generate default theme")?;
        let theme = Theme::from_disk(&paths.get_theme_file()).context("get theme from disk")?;
        let hotkey_map = hotkeys::get_hotkey_config(config.hotkeys.clone());
        let system_instructions = config
            .system
            .instructions
            .first()
            .context("no system instructions")?
            .message
            .clone();
        let mut conversations =
            Self::load_conversations_from_disk(&paths.get_conversations_file())?;
        conversations.insert(0, Conversation::new(system_instructions));

        let ui = ui::Ui::with_provider(config.provider);
        let mut state = Self {
            config,
            models,
            theme,
            hotkey_map,
            paths,
            conversations,
            ui,
        };
        state.set_status_bar_text(format!(
            "Config file: {}",
            state.paths.get_config_file().display()
        ));
        state.add_debug_log("Initialized debug logs");
        Ok(state)
    }

    pub fn reload_config(&mut self) -> Result<()> {
        self.config = Config::from_file(&self.paths.get_config_file(), false)
            .context("get config from file")?;
        self.hotkey_map = hotkeys::get_hotkey_config(self.config.hotkeys.clone());
        self.theme =
            Theme::from_disk(&self.paths.get_theme_file()).context("reload theme from file")?;
        self.set_status_bar_text(format!(
            "Reloaded config file: {}",
            self.paths.get_config_file().display()
        ));
        self.add_debug_log("Reloaded config file.");
        Ok(())
    }

    pub fn reload_models(&mut self) -> Result<()> {
        self.models =
            Models::from_disk(&self.paths.models_dir, false).context("get models from disk")?;
        self.set_status_bar_text(format!(
            "Reloaded model files: {}",
            self.paths.models_dir.display()
        ));
        self.add_debug_log("Reloaded model files.");
        Ok(())
    }

    pub fn ui_clamp(&mut self) -> Result<()> {
        if self.ui.active_conversation_index >= self.conversations.len() {
            self.ui.active_conversation_index = self.conversations.len() - 1;
        }
        if let Some(selected_index) = self.ui.selected_message_index {
            let message_count = self.get_active_conversation()?.messages.len();
            if message_count > 0 && selected_index >= message_count {
                self.ui.selected_message_index = Some(message_count - 1);
            }
        }
        let max_index = self.get_model_count().saturating_sub(1);
        self.ui.selected_model = match self.ui.selected_model {
            SelectedModel::OpenAi(i) => SelectedModel::OpenAi(i.min(max_index)),
            SelectedModel::Anthropic(i) => SelectedModel::Anthropic(i.min(max_index)),
        };
        Ok(())
    }

    pub fn get_active_conversation(&self) -> Result<&Conversation> {
        self.conversations
            .get(self.ui.active_conversation_index)
            .context("active conversation index out of bounds")
    }

    pub fn get_active_conversation_mut(&mut self) -> Result<&mut Conversation> {
        self.conversations
            .get_mut(self.ui.active_conversation_index)
            .context("active conversation index out of bounds")
    }

    pub fn select_model(&mut self, next: bool) -> Result<()> {
        let current_index = match self.ui.selected_model {
            SelectedModel::OpenAi(i) | SelectedModel::Anthropic(i) => i,
        };
        let max_index = i64::try_from(self.get_model_count())?;
        let current_index = i64::try_from(current_index)?;
        #[rustfmt::skip]
        let increment = if next { 1 } else { -1 };
        let new_index = (current_index + increment).rem_euclid(max_index);
        let new_index = usize::try_from(new_index)?;
        self.ui.selected_model = match self.ui.selected_model {
            SelectedModel::OpenAi(_) => SelectedModel::OpenAi(new_index),
            SelectedModel::Anthropic(_) => SelectedModel::Anthropic(new_index),
        };
        Ok(())
    }

    fn get_model_count(&self) -> usize {
        match self.ui.selected_model {
            SelectedModel::OpenAi(_) => self.models.openai.len(),
            SelectedModel::Anthropic(_) => self.models.anthropic.len(),
        }
    }

    pub fn get_models_file(&self) -> PathBuf {
        get_models_file(&self.paths.models_dir, self.ui.selected_model.into())
    }

    pub fn set_status_bar_text(&mut self, text: impl Into<String>) {
        self.ui.status_bar_text = text.into();
    }

    pub fn add_debug_log(&mut self, message: impl std::fmt::Display) {
        self.ui
            .debug_logs
            .push(format!("{} | {message}", crate::get_timestamp()));
    }

    pub fn save_conversations_to_disk(&self) -> Result<()> {
        let data =
            serde_json::to_string_pretty(&self.conversations).context("serialize conversations")?;
        let save_file_path = self.paths.data_dir.join("conversations.json");
        let mut file =
            std::fs::File::create(save_file_path).context("create conversations file")?;
        file.write_all(data.as_bytes())
            .context("write conversations file")?;
        Ok(())
    }

    pub fn load_conversations_from_disk(save_file_path: &PathBuf) -> Result<Vec<Conversation>> {
        if save_file_path.is_file() {
            let data =
                std::fs::read_to_string(save_file_path).context("read conversations file")?;
            let conversations = serde_json::from_str(&data).context("deserialize conversations")?;
            Ok(conversations)
        } else {
            Ok(Vec::new())
        }
    }
}
