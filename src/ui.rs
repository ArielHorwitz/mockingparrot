use crate::State;
use ratatui::{
    prelude::{Constraint, Direction, Layout, Rect, Stylize},
    style::Color,
    widgets::{Block, Paragraph, Wrap},
    Frame,
};
use tui_textarea::TextArea;

pub fn draw_ui_frame(frame: &mut Frame, state: &State, textarea: &TextArea, frame_count: u64) {
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
            .title(crate::APP_TITLE_FULL)
            .bg(Color::Blue)
            .fg(Color::LightGreen)
            .bold(),
        layout[0],
    );

    // Status bar
    frame.render_widget(
        Paragraph::new(format!("[{frame_count}] {}", state.status_bar_text))
            .bg(Color::DarkGray)
            .fg(Color::LightGreen),
        layout[2],
    );

    // Main UI
    draw_main(frame, layout[1], state, textarea);
}

pub fn draw_main(frame: &mut Frame, rect: Rect, state: &State, textarea: &TextArea) {
    let layout = Layout::new(
        Direction::Vertical,
        [Constraint::Fill(1), Constraint::Length(10)],
    )
    .split(rect);
    let main_layout = Layout::new(
        Direction::Horizontal,
        [Constraint::Fill(1), Constraint::Length(30)],
    )
    .split(layout[0]);

    // Conversation display
    frame.render_widget(
        Paragraph::new(format!("{}", state.conversation))
            .wrap(Wrap { trim: false })
            .bg(Color::from_hsl(270.0, 100.0, 5.0))
            .fg(Color::Green),
        main_layout[0],
    );

    // Feedback display
    frame.render_widget(
        Paragraph::new(format!("{:#?}", state.config.chat))
            .wrap(Wrap { trim: false })
            .bg(Color::from_hsl(330.0, 100.0, 5.0))
            .fg(Color::Green),
        main_layout[1],
    );

    // Text input
    frame.render_widget(textarea.widget(), layout[1]);
}
