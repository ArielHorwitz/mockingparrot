use crate::State;
use ratatui::{
    prelude::{Constraint, Direction, Layout, Rect, Stylize},
    style::Color,
    widgets::{Block, Paragraph, Wrap},
    Frame,
};
use tui_textarea::TextArea;

pub fn draw_ui_frame(frame: &mut Frame, state: &State, textarea: &TextArea) {
    let layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Length(1),
            Constraint::Min(4),
            Constraint::Length(1),
        ],
    )
    .split(frame.size());

    // Title bar
    frame.render_widget(
        Block::new()
            .title("HummingParrot AI Chat Client")
            .bg(Color::Blue)
            .fg(Color::LightGreen)
            .bold(),
        layout[0],
    );

    // Status bar
    frame.render_widget(
        Paragraph::new(state.status_bar_text.as_str())
            .bg(Color::DarkGray)
            .fg(Color::LightGreen),
        layout[2],
    );

    // Main UI
    draw_main(frame, layout[1], state, textarea);
}

pub fn draw_main(frame: &mut Frame, rect: Rect, state: &State, textarea: &TextArea) {
    let layout = Layout::new(
        Direction::Horizontal,
        [Constraint::Length(20), Constraint::Fill(1)],
    )
    .split(rect);

    frame.render_widget(textarea.widget(), layout[0]);

    frame.render_widget(
        Paragraph::new(format!("{}\n\n{:#?}", state.feedback, state.config))
            .wrap(Wrap { trim: false })
            .bg(Color::from_hsl(330.0, 100.0, 5.0))
            .fg(Color::Green),
        layout[1],
    );
}
