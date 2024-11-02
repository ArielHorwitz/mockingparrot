use crate::app::{focus::Chat as ChatFocus, state::State};
use anyhow::{Context, Result};
use ratatui::{
    prelude::{Constraint, Direction, Layout, Line, Rect, Style, Stylize, Text},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};

mod history;
mod new;

pub fn draw(frame: &mut Frame, rect: Rect, state: &mut State, scope: ChatFocus) -> Result<()> {
    match scope {
        ChatFocus::New => new::draw(frame, rect, state),
        ChatFocus::History => history::draw(frame, rect, state),
        _ => draw_conversation(frame, rect, state, scope)?,
    };
    Ok(())
}

pub fn draw_conversation(
    frame: &mut Frame,
    rect: Rect,
    state: &mut State,
    scope: ChatFocus,
) -> Result<()> {
    let layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Fill(1),
            Constraint::Length(state.config.ui.layout.prompt_size.saturating_add(2).max(3)),
        ],
    )
    .split(rect);
    let convo_layout = *layout.first().context("ui index")?;
    let prompt_layout = *layout.get(1).context("ui index")?;
    draw_conversation_prompt(frame, prompt_layout, state, scope);
    // Styles
    let is_focused = scope == ChatFocus::Messages;
    let text_color = state.config.ui.colors.text.get_active(is_focused);

    // Conversation display
    let config_file_str = state.paths.get_config_file().display().to_string();
    let missing_api_key = match state.config.provider {
        crate::api::Provider::OpenAi => state.config.keys.openai.is_empty(),
        crate::api::Provider::Anthropic => state.config.keys.anthropic.is_empty(),
    };
    let convo = if missing_api_key {
        Text::from_iter([
            "Missing API key"
                .fg(state.config.ui.colors.text.warn)
                .into(),
            Line::default(),
            "Enter your API key in your config file to start chatting:"
                .fg(text_color)
                .into(),
            config_file_str
                .fg(state.config.ui.colors.text.highlight)
                .into(),
        ])
    } else {
        let active_conversation = &state.get_active_conversation()?;
        let mut lines: Vec<Line> = vec!["System"
            .fg(state.config.ui.colors.text.highlight)
            .underlined()
            .into()];
        for line in active_conversation.system_instructions.lines() {
            lines.push(line.to_owned().fg(text_color).into());
        }
        for message in &state.get_active_conversation()?.messages {
            lines.push(
                format!("{}:", message.role)
                    .to_string()
                    .fg(state.config.ui.colors.text.highlight)
                    .into(),
            );
            for line in message.content.lines() {
                lines.push(line.to_owned().fg(text_color).into());
            }
        }
        Text::from_iter(lines)
    };

    let block = Block::new()
        .borders(Borders::ALL)
        .fg(state.config.ui.colors.frame.get_active(is_focused))
        .title("Conversation")
        .title_style(Style::new().fg(state.config.ui.colors.frame.title));
    let convo_text = Paragraph::new(convo)
        .wrap(Wrap { trim: false })
        .block(block);

    let line_count = convo_text.line_count(convo_layout.width.saturating_sub(2));
    let max_scroll = u16::try_from(line_count)
        .unwrap_or(u16::MAX)
        .saturating_sub(3);
    state.ui.conversation_scroll = state.ui.conversation_scroll.min(max_scroll);
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .style(Style::new().fg(state.config.ui.colors.widget.get_active(is_focused)));
    let mut scrollbar_state =
        ScrollbarState::new(max_scroll.into()).position(state.ui.conversation_scroll.into());
    let scrollbar_area = convo_layout.inner(ratatui::layout::Margin {
        horizontal: 0,
        vertical: 1,
    });

    frame.render_widget(
        convo_text.scroll((state.ui.conversation_scroll, 0)),
        convo_layout,
    );
    frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    Ok(())
}

fn draw_conversation_prompt(frame: &mut Frame, rect: Rect, state: &mut State, scope: ChatFocus) {
    let is_focused = scope == ChatFocus::Prompt;
    let cursor_style = Style::new().bg(state.config.ui.colors.cursor.get_active(is_focused));
    let text_style = Style::new().fg(state.config.ui.colors.text.get_active(is_focused));
    let frame_style = Style::new().fg(state.config.ui.colors.frame.get_active(is_focused));
    let frame_title_style = Style::new().fg(state.config.ui.colors.frame.title);

    state.ui.prompt_textarea.set_cursor_line_style(Style::new());
    state.ui.prompt_textarea.set_cursor_style(cursor_style);
    state.ui.prompt_textarea.set_style(text_style);
    let block = Block::new()
        .borders(Borders::ALL)
        .style(frame_style)
        .title("Prompt")
        .title_style(frame_title_style);
    let inner = block.inner(rect);
    frame.render_widget(block, rect);
    frame.render_widget(&state.ui.prompt_textarea, inner);
}
