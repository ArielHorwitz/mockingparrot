use crate::state::focus::{Conversation as ConversationFocus, Scope};
use crate::state::State;
use anyhow::{Context, Result};
use ratatui::{
    layout::Margin,
    prelude::{Constraint, Direction, Layout, Line, Rect, Style, Stylize, Text},
    style::Color,
    widgets::{
        Block, Borders, List, ListState, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Wrap,
    },
    Frame,
};

const STATUSBAR_HELP_TEXT: &str = "Ctrl+q - Quit, F1 - conversation, F2 - config, F3 - debug";

pub fn draw_frame(frame: &mut Frame, state: &mut State) -> Result<()> {
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
    state: &mut State,
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
    let (convo_block_style, prompt_style, cursor_style) = match conversation_scope {
        ConversationFocus::History => (
            Style::new().fg(state.config.ui.colors.conversation.foreground),
            Style::new()
                .bg(state.config.ui.colors.prompt.background)
                .fg(state.config.ui.colors.prompt.foreground)
                .dim(),
            Style::new(),
        ),
        ConversationFocus::Prompt => (
            Style::new()
                .fg(state.config.ui.colors.conversation.foreground)
                .dim(),
            Style::new()
                .bg(state.config.ui.colors.prompt.background)
                .fg(state.config.ui.colors.prompt.foreground),
            Style::new().bg(Color::Rgb(200, 200, 200)),
        ),
    };

    // Conversation display
    let convo_text_style = Style::new().fg(state.config.ui.colors.conversation.foreground);
    let convo_name_style = Style::new()
        .fg(state.config.ui.colors.conversation_names)
        .bold();
    let convo = if state.config.api.key.is_empty() {
        Text::from_iter([
            Line::styled("Missing API key", Style::new().fg(Color::LightRed)),
            Line::default(),
            Line::styled(
                "Enter your API key in your config file to start chatting:",
                convo_text_style,
            ),
            Line::styled(
                state.paths.config_file.to_string_lossy(),
                Style::new().fg(Color::Yellow).bold(),
            ),
        ])
    } else {
        let mut lines = Vec::new();
        for message in &state.conversation.messages {
            lines.push(Line::styled(
                format!("{:?}:", message.role),
                convo_name_style,
            ));
            for line in message.content.lines() {
                lines.push(Line::styled(line, convo_text_style));
            }
        }
        Text::from_iter(lines)
    };

    let convo_block = Block::new()
        .borders(Borders::ALL)
        .style(convo_block_style)
        .title("Conversation")
        .title_style(convo_block_style);
    let convo_text = Paragraph::new(convo)
        .wrap(Wrap { trim: false })
        .bg(state.config.ui.colors.conversation.background)
        .scroll((state.ui.conversation_scroll, 0))
        .block(convo_block);

    let line_count = convo_text.line_count(convo_layout.width - 2);
    let max_scroll = u16::try_from(line_count).unwrap_or(u16::MAX);
    state.ui.conversation_scroll = state.ui.conversation_scroll.min(max_scroll);
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
    let mut scrollbar_state =
        ScrollbarState::new(max_scroll as usize).position(state.ui.conversation_scroll as usize);
    let scrollbar_area = convo_layout.inner(&ratatui::layout::Margin {
        horizontal: 0,
        vertical: 1,
    });

    frame.render_widget(convo_text, convo_layout);
    frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);

    // Prompt text input
    state.ui.prompt_textarea.set_cursor_line_style(Style::new());
    state.ui.prompt_textarea.set_cursor_style(cursor_style);
    state.ui.prompt_textarea.set_style(prompt_style);
    let prompt_block = Block::new()
        .borders(Borders::ALL)
        .style(prompt_style)
        .title("Prompt")
        .title_style(prompt_style);
    let prompt_area = prompt_block.inner(prompt_layout);
    frame.render_widget(prompt_block, prompt_layout);
    frame.render_widget(state.ui.prompt_textarea.widget(), prompt_area);
    Ok(())
}

fn new_conversation(frame: &mut Frame, rect: Rect, state: &mut State) {
    let list_items = state
        .config
        .system
        .instructions
        .iter()
        .map(|i| format!(">> {}\n{}", i.name, i.message));
    let list = List::new(list_items).highlight_style(Color::LightGreen);
    let mut list_state =
        ListState::default().with_selected(Some(state.ui.system_instruction_selection));
    let block = Block::new()
        .borders(Borders::ALL)
        .border_style(Color::LightCyan)
        .title("New conversation with system instructions:")
        .title_style(Color::LightCyan);
    let list_area = block.inner(rect);
    frame.render_widget(block, rect);
    frame.render_stateful_widget(list, list_area, &mut list_state);
}

fn draw_config(frame: &mut Frame, rect: Rect, state: &mut State) {
    frame.render_widget(
        Paragraph::new(format!(
            "Config file: {}\n{:#?}",
            state.paths.config_file.display(),
            state.config.chat
        ))
        .wrap(Wrap { trim: false })
        .bg(state.config.ui.colors.config.background)
        .fg(state.config.ui.colors.config.foreground),
        rect,
    );
}

fn draw_debug(frame: &mut Frame, rect: Rect, state: &mut State) {
    let debug_logs_block = Block::new().title("Debug logs").borders(Borders::ALL);

    let debug_text = Paragraph::new(state.ui.debug_logs.join("\n"))
        .wrap(Wrap { trim: false })
        .scroll((state.ui.debug_logs_scroll, 0))
        .bg(state.config.ui.colors.debug.background)
        .fg(state.config.ui.colors.debug.foreground)
        .block(debug_logs_block);

    let line_count = debug_text.line_count(rect.width - 2);
    let max_scroll = u16::try_from(line_count).unwrap_or(u16::MAX);
    state.ui.debug_logs_scroll = state.ui.debug_logs_scroll.min(max_scroll);
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
    let mut scrollbar_state =
        ScrollbarState::new(max_scroll as usize).position(state.ui.debug_logs_scroll as usize);
    let scrollbar_area = rect.inner(&Margin {
        horizontal: 1,
        vertical: 1,
    });

    frame.render_widget(debug_text, rect);
    frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
}
