use strum::{EnumIter, IntoEnumIterator};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Focus {
    pub tab: Tab,
    pub chat: Chat,
    pub config: Config,
}

impl Focus {
    #[must_use]
    pub fn get_scope(&self) -> Scope {
        match self.tab {
            Tab::Chat => Scope::Chat(self.chat),
            Tab::Config => Scope::Config(self.config),
            Tab::Debug => Scope::Debug,
        }
    }

    pub fn set_tab(&mut self, tab: Tab) {
        self.tab = tab;
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn cycle_tab_next(&mut self) {
        let tabs = Tab::iter().collect::<Vec<Tab>>();
        let pos = tabs
            .iter()
            .position(|x| *x == self.tab)
            .expect("missing in tab enum");
        self.tab = *tabs.get((pos + 1) % tabs.len()).expect("get next tab");
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn cycle_tab_prev(&mut self) {
        let tabs = Tab::iter().collect::<Vec<Tab>>();
        let pos = tabs
            .iter()
            .position(|x| *x == self.tab)
            .expect("missing in tab enum");
        self.tab = *tabs.get((pos - 1) % tabs.len()).expect("get prev tab");
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn cycle_config_next(&mut self) {
        let configs = Config::iter().collect::<Vec<_>>();
        let pos = configs
            .iter()
            .position(|x| *x == self.config)
            .expect("missing in config enum");
        self.config = *configs
            .get((pos + 1) % configs.len())
            .expect("get next config");
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn cycle_config_prev(&mut self) {
        let configs = Config::iter().collect::<Vec<_>>();
        let pos = configs
            .iter()
            .position(|x| *x == self.config)
            .expect("missing in config enum");
        self.config = *configs
            .get((pos - 1) % configs.len())
            .expect("get prev config");
    }
}

impl Default for Focus {
    fn default() -> Self {
        Self {
            tab: Tab::Chat,
            chat: Chat::Messages,
            config: Config::OpenAi,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, EnumIter)]
pub enum Tab {
    Chat,
    Config,
    Debug,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Chat {
    Messages,
    Prompt,
    New,
    History,
}

#[derive(Debug, PartialEq, Clone, Copy, EnumIter)]
pub enum Config {
    OpenAi,
    Anthropic,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Scope {
    Chat(Chat),
    Config(Config),
    Debug,
}
