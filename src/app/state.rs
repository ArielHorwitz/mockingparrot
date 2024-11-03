use crate::{
    app::hotkeys,
    chat::Conversation,
    config::{Config, Models},
};
use anyhow::{Context, Result};
use std::io::Write;
use std::path::PathBuf;

mod paths;
mod ui;

pub use paths::Paths;

pub struct State {
    pub config: Config,
    pub models: Models,
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

    pub fn fix_clamp_ui_selections(&mut self) {
        if self.ui.active_conversation_index >= self.conversations.len() {
            self.ui.active_conversation_index = self.conversations.len() - 1;
        }
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

    pub fn set_status_bar_text<T: Into<String>>(&mut self, text: T) {
        self.ui.status_bar_text = text.into();
    }

    pub fn add_debug_log<T: Into<String>>(&mut self, log: T) {
        self.ui
            .debug_logs
            .push(format!("{} | {}", crate::get_timestamp(), log.into()));
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
