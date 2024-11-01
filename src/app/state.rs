use crate::{
    app::focus::Focus,
    app::hotkeys,
    chat::Conversation,
    config::{Config, Models},
};
use anyhow::{Context, Result};
use std::io::Write;
use std::path::PathBuf;
use tui_textarea::TextArea;

pub struct State {
    pub config: Config,
    pub models: Models,
    pub hotkey_map: hotkeys::HotkeyMap,
    pub paths: Paths,
    pub conversations: Vec<Conversation>,
    pub ui: Ui,
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

        let ui = Ui {
            focus: Focus::default(),
            status_bar_text: format!("Config file: {}", paths.get_config_file().display()),
            prompt_textarea: TextArea::default(),
            conversation_scroll: 0,
            debug_logs: Vec::new(),
            debug_logs_scroll: 0,
            active_conversation_index: 0,
            system_instruction_selection: 0,
        };
        let mut state = Self {
            config,
            models,
            hotkey_map,
            paths,
            conversations,
            ui,
        };
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

pub struct Paths {
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
    pub models_dir: PathBuf,
}

impl Paths {
    fn generate_dirs() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .context("get config directory")?
            .join(crate::APP_TITLE.to_lowercase());
        let models_dir = config_dir.join("models");
        let data_dir = dirs::data_dir()
            .context("get data directory")?
            .join(crate::APP_TITLE.to_lowercase());
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir).context("create config directory")?;
        }
        if !models_dir.exists() {
            std::fs::create_dir_all(&models_dir).context("create models directory")?;
        }
        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir).context("create data directory")?;
        }
        Ok(Self {
            data_dir,
            config_dir,
            models_dir,
        })
    }

    #[must_use]
    pub fn get_config_file(&self) -> PathBuf {
        self.config_dir.join("config.toml")
    }

    #[must_use]
    pub fn get_message_file(&self) -> PathBuf {
        self.data_dir.join("message_text")
    }

    #[must_use]
    pub fn get_conversations_file(&self) -> PathBuf {
        self.data_dir.join("conversations.json")
    }
}

pub struct Ui {
    pub focus: Focus,
    pub status_bar_text: String,
    pub prompt_textarea: TextArea<'static>,
    pub conversation_scroll: u16,
    pub debug_logs: Vec<String>,
    pub debug_logs_scroll: u16,
    pub active_conversation_index: usize,
    pub system_instruction_selection: usize,
}
