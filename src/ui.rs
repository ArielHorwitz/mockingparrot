use crate::State;
use ratatui::{
    prelude::{Constraint, Direction, Layout, Rect, Style, Stylize},
    style::Color,
    widgets::{Block, Paragraph, Wrap},
    Frame,
};
use tui_textarea::TextArea;


#[derive(Debug)]
pub struct UiState<'a> {
    pub debug: bool,
    pub textarea: TextArea<'a>,
}

impl<'a> Default for UiState<'a> {
    fn default() -> Self {
        Self { debug: false, textarea: get_textarea() }
    }
}

pub fn draw_ui_frame(frame: &mut Frame, state: &State, ui_state: &UiState) {
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
    let now = {
        let now = chrono::Local::now();
        if ui_state.debug {
            format!("{}", now.format("%Y-%m-%d %H:%M:%S%.3f"))
        } else {
            format!("{}", now.format("%Y-%m-%d %H:%M:%S"))
        }
    };
    frame.render_widget(
        Paragraph::new(format!("{now} | {}", state.status_bar_text))
            .bg(Color::DarkGray)
            .fg(Color::LightGreen),
        layout[2],
    );

    // Main UI
    draw_main(frame, layout[1], state, ui_state);
}

pub fn draw_main(frame: &mut Frame, rect: Rect, state: &State, ui_state: &UiState) {
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
    let convo = if state.config.api.key.is_empty() {
        "MISSING API KEY!\n\nEnter an API key in your `config.toml` file to start chatting...\n\nhttps://platform.openai.com".to_owned()
    } else {
        format!("{}", state.conversation)
    };
    frame.render_widget(
        Paragraph::new(convo)
            .wrap(Wrap { trim: false })
            .bg(Color::Rgb(0, 0, 35))
            .fg(Color::Rgb(0, 255, 0)),
        main_layout[0],
    );

    // Feedback display
    frame.render_widget(
        Paragraph::new(format!("{:#?}", state.config.chat))
            .wrap(Wrap { trim: false })
            .bg(Color::Rgb(30, 0, 35))
            .fg(Color::Rgb(125, 150, 0)),
        main_layout[1],
    );

    // Text input
    frame.render_widget(ui_state.textarea.widget(), layout[1]);
}

pub fn get_textarea() -> TextArea<'static> {
    let mut textarea = TextArea::default();
    textarea.set_style(Style::new().bg(Color::Rgb(0, 25, 25)).fg(Color::White));
    textarea.set_line_number_style(Style::new().bg(Color::Black).fg(Color::Cyan));
    textarea.set_cursor_style(Style::new().bg(Color::Rgb(200, 200, 200)));
    textarea
}
