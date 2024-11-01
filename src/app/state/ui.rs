use crate::app::focus::Focus;
use tui_textarea::TextArea;

#[derive(Default)]
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
