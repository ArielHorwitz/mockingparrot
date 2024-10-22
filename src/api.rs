use serde::{Deserialize, Serialize};

mod openai;

pub use openai::get_completion;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Provider {
    OpenAi,
    Anthropic,
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenAi => write!(f, "OpenAI"),
            Self::Anthropic => write!(f, "Anthropic"),
        }
    }
}
