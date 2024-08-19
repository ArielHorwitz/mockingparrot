use crate::config::ValueRange;
use crate::state::{
    focus::{Config as ConfigFocus, Conversation as ConversationFocus, Scope},
    State,
};
use anyhow::{Context, Result};
use ratatui::{
    layout::Margin,
    prelude::{Constraint, Direction, Layout, Line, Rect, Style, Stylize, Text},
    widgets::{
        Block, Borders, List, ListState, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Wrap,
    },
    Frame,
};

pub fn draw_frame(frame: &mut Frame, state: &mut State) -> Result<()> {
    frame.render_widget(
        Block::new().bg(state.config.ui.colors.background.normal),
        frame.area(),
    );
    let layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ],
    )
    .split(frame.area());
    let title_layout = *layout.first().context("ui index")?;
    let main_layout = *layout.get(1).context("ui index")?;
    let status_bar_layout = *layout.get(2).context("ui index")?;

    // Title bar
    frame.render_widget(
        Block::new()
            .title(crate::APP_TITLE_FULL)
            .title_alignment(ratatui::layout::Alignment::Center)
            .fg(state.config.ui.colors.text.title)
            .bold(),
        title_layout,
    );

    // Status bar
    frame.render_widget(
        Paragraph::new(state.ui.status_bar_text.as_str())
            .bg(state.config.ui.colors.background.highlight)
            .fg(state.config.ui.colors.text.normal),
        status_bar_layout,
    );

    // Main UI
    match state.ui.focus.get_scope() {
        Scope::Conversation(conversation_scope) => {
            draw_conversation(frame, main_layout, state, conversation_scope)
                .context("draw conversation tab")?;
        }
        Scope::ConversationHistory => draw_conversation_history(frame, main_layout, state),
        Scope::NewConversation => draw_new_conversation(frame, main_layout, state),
        Scope::Config(config_scope) => {
            draw_config(frame, main_layout, state, config_scope).context("draw config")?;
        }
        Scope::Debug => draw_debug(frame, main_layout, state),
    };
    Ok(())
}

fn draw_conversation_history(frame: &mut Frame, rect: Rect, state: &mut State) {
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
        .title("All conversations")
        .title_style(state.config.ui.colors.frame.title);
    let list_area = block.inner(rect);
    frame.render_widget(block, rect);
    frame.render_stateful_widget(list, list_area, &mut list_state);
}

fn draw_conversation(
    frame: &mut Frame,
    rect: Rect,
    state: &mut State,
    conversation_scope: ConversationFocus,
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
    draw_conversation_prompt(frame, prompt_layout, state, conversation_scope);
    // Styles
    let is_focused = conversation_scope == ConversationFocus::Messages;
    let text_color = state.config.ui.colors.text.get_active(is_focused);

    // Conversation display
    let config_file_str = state.paths.config_file.to_string_lossy();
    let convo = if state.config.api.key.is_empty() {
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
        let mut lines: Vec<Line> = Vec::new();
        for message in &state.get_active_conversation()?.messages {
            lines.push(
                format!("{:?}:", message.role)
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
        .scroll((state.ui.conversation_scroll, 0))
        .block(block);

    let line_count = convo_text.line_count(convo_layout.width - 2);
    let max_scroll = u16::try_from(line_count).unwrap_or(u16::MAX);
    state.ui.conversation_scroll = state.ui.conversation_scroll.min(max_scroll);
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .style(Style::new().fg(state.config.ui.colors.widget.get_active(is_focused)));
    let mut scrollbar_state =
        ScrollbarState::new(max_scroll as usize).position(state.ui.conversation_scroll as usize);
    let scrollbar_area = convo_layout.inner(ratatui::layout::Margin {
        horizontal: 0,
        vertical: 1,
    });

    frame.render_widget(convo_text, convo_layout);
    frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    Ok(())
}

fn draw_conversation_prompt(
    frame: &mut Frame,
    rect: Rect,
    state: &mut State,
    conversation_scope: ConversationFocus,
) {
    let is_focused = conversation_scope == ConversationFocus::Prompt;
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
    frame.render_widget(state.ui.prompt_textarea.widget(), inner);
}

fn draw_new_conversation(frame: &mut Frame, rect: Rect, state: &mut State) {
    let list_items = state
        .config
        .system
        .instructions
        .iter()
        .enumerate()
        .map(|(i, instruction)| {
            let preview = instruction.preview(rect.width.into());
            let (name_style, text_style) = if i == state.ui.system_instruction_selection {
                (
                    Style::new().fg(state.config.ui.colors.text.title).bold(),
                    Style::new().fg(state.config.ui.colors.text.highlight),
                )
            } else {
                (
                    Style::new().fg(state.config.ui.colors.text.title),
                    Style::new().fg(state.config.ui.colors.text.normal),
                )
            };
            Line::from_iter([
                ratatui::prelude::Span::styled(&instruction.name, name_style),
                " | ".fg(state.config.ui.colors.text.normal),
                ratatui::prelude::Span::styled(preview, text_style),
            ])
        });
    let list = List::new(list_items);
    let mut list_state =
        ListState::default().with_selected(Some(state.ui.system_instruction_selection));
    let block = Block::new()
        .borders(Borders::ALL)
        .border_style(state.config.ui.colors.frame.normal)
        .title("New conversation with system instructions:")
        .title_style(state.config.ui.colors.frame.title);
    let list_area = block.inner(rect);
    frame.render_widget(block, rect);
    frame.render_stateful_widget(list, list_area, &mut list_state);
}

fn draw_config(
    frame: &mut Frame,
    rect: Rect,
    state: &mut State,
    config_scope: ConfigFocus,
) -> Result<()> {
    let block = Block::new()
        .borders(Borders::ALL)
        .border_style(state.config.ui.colors.frame.normal)
        .title("Configuration")
        .title_style(state.config.ui.colors.frame.title);

    let layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ],
    )
    .split(block.inner(rect));
    let mut areas_iter = layout.iter();

    let text_style = Style::new().fg(state.config.ui.colors.text.normal);

    frame.render_widget(block, rect);
    frame.render_widget(
        Paragraph::new(format!(
            "Config file: {}",
            state.paths.config_file.display()
        ))
        .style(text_style),
        *areas_iter.next().context("ui index")?,
    );
    frame.render_widget(
        Paragraph::new(format!("Model: {}", state.config.chat.model)).style(text_style),
        *areas_iter.next().context("ui index")?,
    );

    let area = *areas_iter.next().context("ui index")?;
    draw_config_range(
        frame,
        state,
        area,
        &state.config.chat.max_tokens,
        ConfigFocus::MaxTokens,
        config_scope,
    );
    for (range_type, value_range) in [
        (ConfigFocus::Temperature, state.config.chat.temperature),
        (ConfigFocus::TopP, state.config.chat.top_p),
        (
            ConfigFocus::FrequencyPenalty,
            state.config.chat.frequency_penalty,
        ),
        (
            ConfigFocus::PresencePenalty,
            state.config.chat.presence_penalty,
        ),
    ] {
        let area = *areas_iter.next().context("ui index")?;
        draw_config_range(frame, state, area, &value_range, range_type, config_scope);
    }
    Ok(())
}

fn draw_config_range<T: std::fmt::Debug + std::fmt::Display>(
    frame: &mut Frame,
    state: &State,
    rect: Rect,
    value_range: &ValueRange<T>,
    value_range_type: ConfigFocus,
    config_scope: ConfigFocus,
) {
    let is_focused = value_range_type == config_scope;
    let color = state.config.ui.colors.text.get_highlight(is_focused);
    let text = format!(
        "{:?}: {} ({} - {})",
        value_range_type, value_range.value, value_range.min, value_range.max,
    );
    frame.render_widget(Paragraph::new(text).fg(color), rect);
}

fn draw_debug(frame: &mut Frame, rect: Rect, state: &mut State) {
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
