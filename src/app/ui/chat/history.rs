use crate::app::state::State;
use ratatui::{
    prelude::Rect,
    widgets::{Block, Borders, List, ListState},
    Frame,
};

pub fn draw(frame: &mut Frame, rect: Rect, state: &mut State) {
    let list_items = state
        .conversations
        .iter()
        .map(|c| c.preview(rect.width.into()));
    let list = List::new(list_items)
        .style(state.config.ui.colors.text.normal)
        .highlight_style(state.config.ui.colors.text.highlight);
    let mut list_state =
        ListState::default().with_selected(Some(state.ui.active_conversation_index));
    let block = Block::new()
        .borders(Borders::ALL)
        .border_style(state.config.ui.colors.frame.normal)
        .title("Load conversation from history:")
        .title_style(state.config.ui.colors.frame.title);
    let list_area = block.inner(rect);
    frame.render_widget(block, rect);
    frame.render_stateful_widget(list, list_area, &mut list_state);
}
