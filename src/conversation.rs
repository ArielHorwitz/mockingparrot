use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    System,
    Assistant,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    #[must_use]
    pub fn new_user_message(content: String) -> Self {
        Self {
            role: Role::User,
            content,
        }
    }

    #[must_use]
    pub fn new_system_message(content: String) -> Self {
        Self {
            role: Role::System,
            content,
        }
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.role, self.content)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub messages: Vec<Message>,
}

impl Conversation {
    #[must_use]
    pub fn new(system_instructions: String) -> Self {
        Self {
            messages: vec![Message::new_system_message(system_instructions)],
        }
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.messages.get(1).is_none()
    }

    #[must_use]
    pub fn preview(&self, length: usize) -> String {
        if let Some(first_message) = self.messages.get(1) {
            first_message
                .content
                .chars()
                .take(length)
                .map(|char| if char == '\n' { ' ' } else { char })
                .collect()
        } else if let Some(system_instructions) = self.messages.first() {
            let content: String = system_instructions
                .content
                .chars()
                .take(length.saturating_sub(6))
                .map(|char| if char == '\n' { ' ' } else { char })
                .collect();
            format!("<NEW> {content}")
        } else {
            "<EMPTY>".to_string()
        }
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
