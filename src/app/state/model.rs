use crate::api::Provider;

#[derive(Debug, Clone, Copy)]
pub enum SelectedModel {
    Anthropic(usize),
    OpenAi(usize),
}

impl From<SelectedModel> for Provider {
    fn from(value: SelectedModel) -> Self {
        match value {
            SelectedModel::Anthropic(_) => Self::Anthropic,
            SelectedModel::OpenAi(_) => Self::OpenAi,
        }
    }
}

impl From<Provider> for SelectedModel {
    fn from(value: Provider) -> Self {
        match value {
            Provider::Anthropic => Self::Anthropic(0),
            Provider::OpenAi => Self::OpenAi(0),
        }
    }
}

impl SelectedModel {
    #[must_use]
    pub fn next_provider(self) -> Self {
        match self {
            Self::Anthropic(index) => Self::OpenAi(index),
            Self::OpenAi(index) => Self::Anthropic(index),
        }
    }

    #[must_use]
    pub fn previous_provider(self) -> Self {
        self.next_provider()
    }
}
