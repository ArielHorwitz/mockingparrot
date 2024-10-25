use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct System {
    pub instructions: Vec<SystemInstructions>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Deserialize, Clone)]
pub struct SystemInstructions {
    pub name: String,
    pub message: String,
}

impl SystemInstructions {
    #[must_use]
    pub fn preview(&self, length: usize) -> String {
        self.message
            .chars()
            .take(length)
            .map(|char| if char == '\n' { ' ' } else { char })
            .collect()
    }
}
