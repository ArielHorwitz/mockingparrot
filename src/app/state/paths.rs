use anyhow::{Context, Result};
use std::path::PathBuf;

pub struct Paths {
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
    pub models_dir: PathBuf,
}

impl Paths {
    pub fn generate_dirs() -> Result<Self> {
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
