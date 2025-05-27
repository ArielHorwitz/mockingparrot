use crate::app::focus::Focus;
use crate::app::state::SelectedModel;
use tui_textarea::TextArea;

pub struct Ui {
    pub focus: Focus,
    pub status_bar_text: String,
    pub prompt_textarea: TextArea<'static>,
    pub conversation_scroll: u16,
    pub debug_logs: Vec<String>,
    pub debug_logs_scroll: u16,
    pub active_conversation_index: usize,
    pub selected_message_index: Option<usize>,
    pub system_instruction_selection: usize,
    pub selected_model: SelectedModel,
}

impl Ui {
    #[must_use]
    pub fn with_provider(provider: crate::api::Provider) -> Self {
        Ui {
            focus: Focus::default(),
            status_bar_text: String::new(),
            prompt_textarea: TextArea::default(),
            conversation_scroll: 0,
            debug_logs: Vec::new(),
            debug_logs_scroll: 0,
            active_conversation_index: 0,
            selected_message_index: None,
            system_instruction_selection: 0,
            selected_model: SelectedModel::from(provider),
        }
    }
}
