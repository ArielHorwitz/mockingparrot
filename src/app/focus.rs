use anyhow::Context;
use strum::{EnumIter, IntoEnumIterator};

#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct Focus {
    pub tab: Tab,
    pub chat: Chat,
}

impl Focus {
    #[must_use]
    pub fn get_scope(&self) -> Scope {
        match self.tab {
            Tab::Chat => Scope::Chat(self.chat),
            Tab::Models => Scope::Models,
            Tab::Config => Scope::Config,
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
        bounded
            .checked_add(total)
            .context("add total to negative")?
    } else {
        bounded
    };
    let final_result = usize::try_from(positive).context("usize from result")?;
    Ok(final_result)
}

#[derive(Default, Debug, PartialEq, Clone, Copy, EnumIter)]
pub enum Tab {
    #[default]
    Chat,
    Models,
    Config,
    Debug,
}

#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub enum Chat {
    #[default]
    Messages,
    Prompt,
    New,
    History,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Scope {
    Chat(Chat),
    Models,
    Config,
    Debug,
}
