use crate::app::focus::Focus;
use tui_textarea::TextArea;

pub struct Ui {
    pub focus: Focus,
    pub status_bar_text: String,
    pub prompt_textarea: TextArea<'static>,
    pub conversation_scroll: u16,
    pub debug_logs: Vec<String>,
    pub debug_logs_scroll: u16,
    pub active_conversation_index: usize,
    pub system_instruction_selection: usize,
}

impl Ui {
    #[must_use]
    pub fn with_provider(provider: crate::api::Provider) -> Self {
        Ui {
            focus: Focus::with_provider(provider),
            status_bar_text: String::default(),
            prompt_textarea: TextArea::default(),
            conversation_scroll: Default::default(),
            debug_logs: Vec::default(),
            debug_logs_scroll: Default::default(),
            active_conversation_index: Default::default(),
            system_instruction_selection: Default::default(),
        }
    }
}
