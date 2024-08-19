const TAB_ORDER: [Tab; 3] = [Tab::Conversation, Tab::Config, Tab::Debug];

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
            Tab::Config => Scope::Config(self.config),
            Tab::Debug => Scope::Debug,
        }
    }

    pub fn set_tab(&mut self, tab: Tab) {
        self.tab = tab;
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn cycle_tab_next(&mut self) {
        let len = TAB_ORDER.len();
        let pos = TAB_ORDER
            .iter()
            .position(|x| *x == self.tab)
            .expect("missing in tab order");
        self.tab = TAB_ORDER[(pos + 1) % len];
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn cycle_tab_prev(&mut self) {
        let len = TAB_ORDER.len();
        let pos = TAB_ORDER
            .iter()
            .position(|x| *x == self.tab)
            .expect("missing in tab order");
        self.tab = TAB_ORDER[(pos + len - 1) % len];
    }
}

impl Default for Focus {
    fn default() -> Self {
        Self {
            tab: Tab::Conversation,
            conversation: Conversation::Messages,
            config: Config::MaxTokens,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Tab {
    Conversation,
    Config,
    Debug,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Conversation {
    Messages,
    Prompt,
    New,
    History,
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
    Config(Config),
    Debug,
}
