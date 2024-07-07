#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Focus {
    pub tab: Tab,
    pub conversation: Conversation,
}

impl Focus {
    #[must_use]
    pub fn get_scope(&self) -> Scope {
        match self.tab {
            Tab::Conversation => Scope::Conversation(self.conversation),
            Tab::NewConversation => Scope::NewConversation,
            Tab::Config => Scope::Config,
            Tab::Debug => Scope::Debug,
        }
    }

    pub fn set_tab(&mut self, tab: Tab) {
        self.tab = tab;
    }
}

impl Default for Focus {
    fn default() -> Self {
        Self {
            tab: Tab::Conversation,
            conversation: Conversation::History,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Tab {
    Conversation,
    NewConversation,
    Config,
    Debug,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Conversation {
    History,
    Prompt,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Scope {
    Conversation(Conversation),
    NewConversation,
    Config,
    Debug,
}
