use crate::state::focus::{Conversation as ConversationFocus, Scope};
use crate::state::State;
use anyhow::{Context, Result};
use ratatui::{
    prelude::{Constraint, Direction, Layout, Rect, Style, Stylize},
    style::Color,
    widgets::{Block, Borders, List, ListState, Paragraph, Wrap},
    Frame,
};

const STATUSBAR_HELP_TEXT: &str = "Ctrl+q - Quit, F1 - conversation, F2 - config, F3 - debug";

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
        Block::new().title(crate::APP_TITLE_FULL).fg(Color::Green),
        title_layout,
    );

    // Status bar
    let now = format!("{}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
    frame.render_widget(
        Paragraph::new(state.ui.status_bar_text.as_str())
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
    match state.ui.focus.get_scope() {
        Scope::Conversation(conversation_scope) => {
            draw_conversation(frame, main_layout, state, conversation_scope)
                .context("draw conversation tab")?;
        }
        Scope::NewConversation => new_conversation(frame, main_layout, state),
        Scope::Config => draw_config(frame, main_layout, state),
        Scope::Debug => draw_debug(frame, main_layout, state),
    };
    Ok(())
}

fn draw_conversation(
    frame: &mut Frame,
    rect: Rect,
    state: &State,
    conversation_scope: ConversationFocus,
) -> Result<()> {
    let layout = Layout::new(
        Direction::Vertical,
        [Constraint::Fill(1), Constraint::Length(10)],
    )
    .split(rect);
    let convo_layout = *layout.first().context("ui index")?;
    let prompt_layout = *layout.get(1).context("ui index")?;

    // Styles for focus
    let (convo_block_style, prompt_block_style) = match conversation_scope {
        ConversationFocus::History => (
            Style::new().fg(Color::LightCyan),
            Style::new().fg(Color::Yellow),
        ),
        ConversationFocus::Prompt => (
            Style::new().fg(Color::Blue),
            Style::new().fg(Color::LightYellow),
        ),
    };

    // Conversation display
    let convo = if state.config.api.key.is_empty() {
        format!("MISSING API KEY!\n\nEnter your API key in your config file to start chatting.\nThe config file is located at: ~/{}", crate::config::CONFIG_FILE_PATH)
    } else {
        state.conversation.to_string()
    };
    let convo_title = format!(
        "Conversation ({}/{})",
        state.ui.conversation_scroll,
        state.conversation.messages.len()
    );

    let convo_block = Block::new()
        .borders(Borders::ALL)
        .style(convo_block_style)
        .title(convo_title)
        .title_style(convo_block_style);

    frame.render_widget(
        Paragraph::new(convo)
            .wrap(Wrap { trim: false })
            .bg(Color::Rgb(0, 0, 35))
            .fg(Color::LightGreen)
            .scroll((state.ui.conversation_scroll, 0))
            .block(convo_block),
        convo_layout,
    );

    // Prompt text input
    let prompt_block = Block::new()
        .borders(Borders::ALL)
        .style(prompt_block_style)
        .title("Prompt:")
        .title_style(prompt_block_style);
    let prompt_area = prompt_block.inner(prompt_layout);
    frame.render_widget(prompt_block, prompt_layout);
    frame.render_widget(state.ui.prompt_textarea.widget(), prompt_area);
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
        ListState::default().with_selected(Some(state.ui.system_instruction_selection));
    frame.render_stateful_widget(list, rect, &mut list_state);
}

fn draw_config(frame: &mut Frame, rect: Rect, state: &State) {
    frame.render_widget(
        Paragraph::new(format!("{:#?}", state.config.chat))
            .wrap(Wrap { trim: false })
            .bg(Color::Rgb(0, 20, 35))
            .fg(Color::Rgb(125, 150, 0)),
        rect,
    );
}

fn draw_debug(frame: &mut Frame, rect: Rect, state: &State) {
    let debug_logs_block = Block::new()
        .title(format!(
            "Debug logs ({}/{})",
            state.ui.debug_logs_scroll,
            state.ui.debug_logs.len()
        ))
        .borders(Borders::TOP | Borders::LEFT);
    frame.render_widget(
        Paragraph::new(state.ui.debug_logs.join("\n"))
            .wrap(Wrap { trim: false })
            .scroll((state.ui.debug_logs_scroll, 0))
            .bg(Color::Rgb(30, 30, 0))
            .fg(Color::Rgb(125, 150, 0))
            .block(debug_logs_block),
        rect,
    );
}
