use crate::app::state::State;
use ratatui::{
    prelude::{Line, Rect, Style, Stylize},
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
                (
                    Style::new().fg(state.config.ui.colors.text.title).bold(),
                    Style::new().fg(state.config.ui.colors.text.highlight),
                )
            } else {
                (
                    Style::new().fg(state.config.ui.colors.text.title),
                    Style::new().fg(state.config.ui.colors.text.normal),
                )
            };
            Line::from_iter([
                ratatui::prelude::Span::styled(&instruction.name, name_style),
                " | ".fg(state.config.ui.colors.text.normal),
                ratatui::prelude::Span::styled(preview, text_style),
            ])
        });
    let list = List::new(list_items);
    let mut list_state =
        ListState::default().with_selected(Some(state.ui.system_instruction_selection));
    let block = Block::new()
        .borders(Borders::ALL)
        .border_style(state.config.ui.colors.frame.normal)
        .title("Start new conversation:")
        .title_style(state.config.ui.colors.frame.title);
    let list_area = block.inner(rect);
    frame.render_widget(block, rect);
    frame.render_stateful_widget(list, list_area, &mut list_state);
}
