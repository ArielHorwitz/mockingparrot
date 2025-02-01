use crate::api::Provider;
use anyhow::Context;
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
        let next_tab = cycle_unsigned(pos, tabs.len(), false).expect("cycle math");
        self.tab = *tabs.get(next_tab).expect("get next tab");
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn cycle_tab_prev(&mut self) {
        let tabs = Tab::iter().collect::<Vec<Tab>>();
        let pos = tabs
            .iter()
            .position(|x| *x == self.tab)
            .expect("missing in tab enum");
        let prev_tab = cycle_unsigned(pos, tabs.len(), true).expect("cycle math");
        self.tab = *tabs.get(prev_tab).expect("get prev tab");
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn cycle_config_next(&mut self) {
        let configs = Config::iter().collect::<Vec<_>>();
        let pos = configs
            .iter()
            .position(|x| *x == self.config)
            .expect("missing in config enum");
        let next_pos = cycle_unsigned(pos, configs.len(), false).expect("cycle math");
        self.config = *configs.get(next_pos).expect("get next config");
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn cycle_config_prev(&mut self) {
        let configs = Config::iter().collect::<Vec<_>>();
        let pos = configs
            .iter()
            .position(|x| *x == self.config)
            .expect("missing in config enum");
        let prev_pos = cycle_unsigned(pos, configs.len(), true).expect("cycle math");
        self.config = *configs.get(prev_pos).expect("get prev config");
    }
}

pub fn cycle_unsigned(current: usize, total: usize, subtract: bool) -> anyhow::Result<usize> {
    let current = i32::try_from(current).context("i32 from current")?;
    let total = i32::try_from(total).context("i32 from total")?;
    let new = if subtract {
        current.checked_sub(1).context("sub 1")?
    } else {
        current.checked_add(1).context("add 1")?
    };
    let bounded = new.checked_rem(total).context("remainder from total")?;
    let positive = if bounded < 0 {
        bounded.checked_add(total).context("add total to negative")?
    } else {
        bounded
    };
    let final_result = usize::try_from(positive).context("usize from result")?;
    Ok(final_result)
}

impl Focus {
    #[must_use]
    pub fn with_provider(provider: Provider) -> Self {
        Self {
            tab: Tab::Chat,
            chat: Chat::Messages,
            config: Config::from_provider(provider),
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

impl Config {
    #[must_use]
    pub fn from_provider(provider: Provider) -> Self {
        match provider {
            Provider::OpenAi => Self::OpenAi,
            Provider::Anthropic => Self::Anthropic,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Scope {
    Chat(Chat),
    Config(Config),
    Debug,
}
