use crate::config::Config;

#[derive(Debug)]
pub struct State {
    pub config: Config,
    pub feedback: String,
    pub status_bar_text: String,
}

impl State {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            feedback: "Welcome to HummingParrot".to_owned(),
            status_bar_text: "Welcome to HummingParrot".to_owned(),
        }
    }
}
