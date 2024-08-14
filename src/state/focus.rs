#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Focus {
    pub tab: Tab,
    pub conversation: Conversation,
    pub config: Config,
}

impl Focus {
    #[must_use]
    pub fn get_scope(&self) -> Scope {
        match self.tab {
            Tab::Conversation => Scope::Conversation(self.conversation),
            Tab::NewConversation => Scope::NewConversation,
            Tab::Config => Scope::Config(self.config),
            Tab::Debug => Scope::Debug,
        }
    }

    pub fn set_tab(&mut self, tab: Tab) {
        self.tab = tab;
    }

    pub fn cycle_tab(&mut self) {
        self.tab = match self.tab {
            Tab::Conversation => Tab::Config,
            Tab::Config => Tab::Debug,
            Tab::Debug | Tab::NewConversation => Tab::Conversation,
        };
    }
}

impl Default for Focus {
    fn default() -> Self {
        Self {
            tab: Tab::Conversation,
            conversation: Conversation::History,
            config: Config::MaxTokens,
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
pub enum Config {
    MaxTokens,
    Temperature,
    TopP,
    FrequencyPenalty,
    PresencePenalty,
}

impl Config {
    #[must_use]
    pub fn next_cycle(&self) -> Config {
        match self {
            Config::MaxTokens => Config::Temperature,
            Config::Temperature => Config::TopP,
            Config::TopP => Config::FrequencyPenalty,
            Config::FrequencyPenalty => Config::PresencePenalty,
            Config::PresencePenalty => Config::MaxTokens,
        }
    }

    #[must_use]
    pub fn prev_cycle(&self) -> Config {
        match self {
            Config::MaxTokens => Config::PresencePenalty,
            Config::Temperature => Config::MaxTokens,
            Config::TopP => Config::Temperature,
            Config::FrequencyPenalty => Config::TopP,
            Config::PresencePenalty => Config::FrequencyPenalty,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Scope {
    Conversation(Conversation),
    NewConversation,
    Config(Config),
    Debug,
}
