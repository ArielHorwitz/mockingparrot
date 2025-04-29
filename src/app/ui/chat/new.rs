use crate::app::state::State;
use ratatui::{
    prelude::{Line, Rect},
    widgets::{Block, Borders, List, ListState},
    Frame,
};

pub fn draw(frame: &mut Frame, rect: Rect, state: &mut State) {
    let list_items = state
        .config
        .system
        .instructions
        .iter()
        .enumerate()
        .map(|(i, instruction)| {
            let preview = instruction.preview(rect.width.into());
            let (name_style, text_style) = if i == state.ui.system_instruction_selection {
                (state.theme.title(), state.theme.text(true))
            } else {
                (state.theme.title(), state.theme.text(false))
            };
            Line::from_iter([
                ratatui::prelude::Span::styled(&instruction.name, name_style),
                ratatui::prelude::Span::styled(" | ", state.theme.text(false)),
                ratatui::prelude::Span::styled(preview, text_style),
            ])
        });
    let list = List::new(list_items);
    let mut list_state =
        ListState::default().with_selected(Some(state.ui.system_instruction_selection));
    let block = Block::new()
        .borders(Borders::ALL)
        .border_style(state.theme.frame(true))
        .title("Start new conversation:")
        .title_style(state.theme.title());
    let list_area = block.inner(rect);
    frame.render_widget(block, rect);
    frame.render_stateful_widget(list, list_area, &mut list_state);
}
