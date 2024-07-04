use crate::state::{State, ViewTab};
use anyhow::{Context, Result};
use ratatui::{
    prelude::{Constraint, Direction, Layout, Rect, Stylize},
    style::Color,
    widgets::{Block, Borders, List, ListState, Paragraph, Wrap},
    Frame,
};

const STATUSBAR_HELP_TEXT: &str = "Ctrl+q - Quit, F1 - conversation, F2 - config/debug";

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
    let title_layout = *layout.first().context("ui index")?;
    let main_layout = *layout.get(1).context("ui index")?;
    let status_bar_layout = *layout.get(2).context("ui index")?;
    let help_bar_layout = *layout.get(3).context("ui index")?;

    // Title bar
    frame.render_widget(
        Block::new()
            .title(crate::APP_TITLE_FULL)
            .bg(Color::Blue)
            .fg(Color::LightGreen)
            .bold(),
        title_layout,
    );

    // Status bar
    let now = format!("{}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
    frame.render_widget(
        Paragraph::new(state.status_bar_text.as_str())
            .bg(Color::DarkGray)
            .fg(Color::LightGreen),
        status_bar_layout,
    );
    frame.render_widget(
        Paragraph::new(format!("{STATUSBAR_HELP_TEXT} | {now}"))
            .bg(Color::Black)
            .fg(Color::Green),
        help_bar_layout,
    );

    // Main UI
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
    let convo_layout = *layout.first().context("ui index")?;
    let prompt_layout = *layout.get(1).context("ui index")?;

    // Conversation display
    let convo = if state.config.api.key.is_empty() {
        format!("MISSING API KEY!\n\nEnter your API key in your config file to start chatting.\nThe config file is located at: ~/{}", crate::config::CONFIG_FILE_PATH)
    } else {
        state.conversation.to_string()
    };
    let convo_title = format!(
        "Conversation ({}/{})",
        state.conversation_scroll,
        state.conversation.messages.len()
    );
    let convo_block = Block::new()
        .borders(Borders::ALL)
        .style(Color::LightBlue)
        .title(convo_title)
        .title_style(Color::LightCyan);
    frame.render_widget(
        Paragraph::new(convo)
            .wrap(Wrap { trim: false })
            .bg(Color::Rgb(0, 0, 35))
            .fg(Color::LightGreen)
            .scroll((state.conversation_scroll, 0))
            .block(convo_block),
        convo_layout,
    );

    // Text input
    let convo_block = Block::new()
        .borders(Borders::ALL)
        .style(Color::Yellow)
        .title("Prompt:")
        .title_style(Color::LightYellow);
    let prompt_area = convo_block.inner(prompt_layout);
    frame.render_widget(convo_block, prompt_layout);
    frame.render_widget(state.prompt_textarea.widget(), prompt_area);
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
    let mut list_state =
        ListState::default().with_selected(Some(state.system_instruction_selection));
    frame.render_stateful_widget(list, rect, &mut list_state);
}

fn draw_config(frame: &mut Frame, rect: Rect, state: &State) -> Result<()> {
    let main_layout = Layout::new(
        Direction::Vertical,
        [Constraint::Length(9), Constraint::Fill(1)],
    )
    .split(rect);
    let debug_config_layout = *main_layout.first().context("ui index")?;
    let debug_logs_layout = *main_layout.get(1).context("ui index")?;

    frame.render_widget(
        Paragraph::new(format!("{:#?}", state.config.chat))
            .wrap(Wrap { trim: false })
            .bg(Color::Rgb(0, 20, 35))
            .fg(Color::Rgb(125, 150, 0)),
        debug_config_layout,
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
        debug_logs_layout,
    );
    Ok(())
}
