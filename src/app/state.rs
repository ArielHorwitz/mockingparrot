use crate::{
    app::focus::Focus,
    app::hotkeys,
    chat::Conversation,
    config::{get_config_from_file, Config},
};
use anyhow::{Context, Result};
use std::io::Write;
use std::path::PathBuf;
use tui_textarea::TextArea;

pub struct State {
    pub config: Config,
    pub hotkey_map: hotkeys::HotkeyMap,
    pub paths: Paths,
    pub conversations: Vec<Conversation>,
    pub ui: Ui,
}

impl State {
    pub fn new() -> Result<Self> {
        let paths = Paths::generate_dirs().context("generate directories")?;
        let config = get_config_from_file(&paths.config_file).context("get config from file")?;
        let hotkey_map = hotkeys::get_hotkey_config(config.hotkeys.clone());
        let system_instructions = config
            .system
            .instructions
            .first()
            .context("no system instructions")?
            .message
            .clone();
        let mut conversations = Self::load_conversations_from_disk(&paths.conversations_file)?;
        conversations.insert(0, Conversation::new(system_instructions));

        let ui = Ui {
            focus: Focus::default(),
            status_bar_text: format!("Config file: {}", paths.config_file.display()),
            prompt_textarea: TextArea::default(),
            conversation_scroll: 0,
            debug_logs: Vec::new(),
            debug_logs_scroll: 0,
            active_conversation_index: 0,
            system_instruction_selection: 0,
        };
        let mut state = Self {
            config,
            hotkey_map,
            paths,
            conversations,
            ui,
        };
        state.add_debug_log("Initialized debug logs");
        Ok(state)
    }

    pub fn reload_config(&mut self) -> Result<()> {
        self.config =
            get_config_from_file(&self.paths.config_file).context("get config from file")?;
        self.hotkey_map = hotkeys::get_hotkey_config(self.config.hotkeys.clone());
        self.set_status_bar_text(format!(
            "Reloaded config file: {}",
            self.paths.config_file.display()
        ));
        self.add_debug_log("Reloaded config file.");
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
    pub config_file: PathBuf,
    pub message_file: PathBuf,
    pub conversations_file: PathBuf,
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
        let conversations_file = data_dir.join("conversations.json");
        Ok(Self {
            data_dir,
            config_dir,
            config_file,
            message_file,
            conversations_file,
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
    pub active_conversation_index: usize,
    pub system_instruction_selection: usize,
}
