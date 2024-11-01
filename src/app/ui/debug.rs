use crate::app::state::State;
use ratatui::{
    layout::Margin,
    prelude::{Rect, Style, Stylize},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};

pub fn draw(frame: &mut Frame, rect: Rect, state: &mut State) {
    let debug_logs_block = Block::new()
        .title("Debug logs")
        .borders(Borders::ALL)
        .fg(state.config.ui.colors.frame.normal)
        .title_style(Style::new().fg(state.config.ui.colors.frame.title));

    let debug_text = Paragraph::new(state.ui.debug_logs.join("\n"))
        .wrap(Wrap { trim: false })
        .scroll((state.ui.debug_logs_scroll, 0))
        .fg(state.config.ui.colors.text.normal)
        .block(debug_logs_block);

    let line_count = debug_text.line_count(rect.width - 2);
    let max_scroll = u16::try_from(line_count).unwrap_or(u16::MAX);
    state.ui.debug_logs_scroll = state.ui.debug_logs_scroll.min(max_scroll);
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .style(Style::new().fg(state.config.ui.colors.widget.normal));
    let mut scrollbar_state =
        ScrollbarState::new(max_scroll as usize).position(state.ui.debug_logs_scroll as usize);
    let scrollbar_area = rect.inner(Margin {
        horizontal: 0,
        vertical: 1,
    });

    frame.render_widget(debug_text, rect);
    frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
}
