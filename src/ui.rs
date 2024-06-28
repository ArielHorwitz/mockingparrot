use crate::state::{State, ViewTab};
use anyhow::{Context, Result};
use ratatui::{
    prelude::{Constraint, Direction, Layout, Rect, Stylize},
    style::Color,
    widgets::{Block, Borders, List, Paragraph, Wrap},
    Frame,
};

const STATUSBAR_HELP_TEXT: &str = "Ctrl+q - Quit, F1 - conversation, F2 - config/debug";

/// Draw the UI frame.
///
/// # Errors
/// An error with a description is returned in case of failure.
pub fn draw_frame(frame: &mut Frame, state: &State) -> Result<()> {
    let layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
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
        *layout.first().context("ui index")?,
    );

    // Status bar
    let now = {
        let now = chrono::Local::now();
        if state.debug {
            format!("{}", now.format("%Y-%m-%d %H:%M:%S%.3f"))
        } else {
            format!("{}", now.format("%Y-%m-%d %H:%M:%S"))
        }
    };
    frame.render_widget(
        Paragraph::new(state.status_bar_text.as_str())
            .bg(Color::DarkGray)
            .fg(Color::LightGreen),
        *layout.get(2).context("ui index")?,
    );
    frame.render_widget(
        Paragraph::new(format!("{STATUSBAR_HELP_TEXT} | {now}"))
            .bg(Color::Black)
            .fg(Color::Green),
        *layout.get(3).context("ui index")?,
    );

    // Main UI
    let main_layout = *layout.get(1).context("ui index")?;
    match state.tab {
        ViewTab::Conversation => {
            draw_conversation(frame, main_layout, state).context("draw conversation tab")?;
        }
        ViewTab::NewConversation => {
            new_conversation(frame, main_layout, state);
        }
        ViewTab::Config => draw_config(frame, main_layout, state).context("draw config tab")?,
    };
    Ok(())
}

fn draw_conversation(frame: &mut Frame, rect: Rect, state: &State) -> Result<()> {
    let layout = Layout::new(
        Direction::Vertical,
        [Constraint::Fill(1), Constraint::Length(10)],
    )
    .split(rect);

    // Conversation display
    let convo = if state.config.api.key.is_empty() {
        "MISSING API KEY!\n\nEnter an API key in your `config.toml` file to start chatting...\n\nhttps://platform.openai.com".to_owned()
    } else {
        state.conversation.to_string()
    };
    frame.render_widget(
        Paragraph::new(convo)
            .wrap(Wrap { trim: false })
            .bg(Color::Rgb(0, 0, 35))
            .fg(Color::Rgb(0, 255, 0)),
        *layout.first().context("ui index")?,
    );

    // Text input
    frame.render_widget(
        state.prompt_textarea.widget(),
        *layout.get(1).context("ui index")?,
    );
    Ok(())
}

fn new_conversation(frame: &mut Frame, rect: Rect, state: &State) {
    let list_items = state
        .config
        .system
        .instructions
        .iter()
        .map(|i| format!(">> {}\n{}", i.name, i.message));
    let list = List::new(list_items).highlight_style(Color::LightGreen);
    let mut list_state = state.system_instruction_selection.clone();
    frame.render_stateful_widget(list, rect, &mut list_state);
}

fn draw_config(frame: &mut Frame, rect: Rect, state: &State) -> Result<()> {
    let main_layout = Layout::new(
        Direction::Vertical,
        [Constraint::Length(9), Constraint::Fill(1)],
    )
    .split(rect);

    frame.render_widget(
        Paragraph::new(format!("{:#?}", state.config.chat))
            .wrap(Wrap { trim: false })
            .bg(Color::Rgb(0, 20, 35))
            .fg(Color::Rgb(125, 150, 0)),
        *main_layout.first().context("ui index")?,
    );
    let debug_logs_block = Block::new()
        .title(format!(
            "Debug logs ({}/{})",
            state.debug_logs_scroll,
            state.debug_logs.len()
        ))
        .borders(Borders::TOP | Borders::LEFT);
    frame.render_widget(
        Paragraph::new(state.debug_logs.join("\n"))
            .wrap(Wrap { trim: false })
            .scroll((state.debug_logs_scroll, 0))
            .bg(Color::Rgb(30, 30, 0))
            .fg(Color::Rgb(125, 150, 0))
            .block(debug_logs_block),
        *main_layout.get(1).context("ui index")?,
    );
    Ok(())
}
