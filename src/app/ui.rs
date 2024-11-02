use crate::app::{focus::Scope, state::State};
use anyhow::{Context, Result};
use ratatui::{
    prelude::{Constraint, Direction, Layout, Stylize},
    widgets::{Block, Paragraph},
    Frame,
};

mod chat;
mod config;
mod debug;

pub fn draw(frame: &mut Frame, state: &mut State) -> Result<()> {
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

    draw_title_tabs(frame, state, title_layout).context("draw title tabs")?;

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
            chat::draw(frame, main_layout, state, conversation_scope).context("draw chat tab")?;
        }
        Scope::Config(config_scope) => {
            config::draw(frame, main_layout, state, config_scope).context("draw config")?;
        }
        Scope::Debug => debug::draw(frame, main_layout, state),
    };
    Ok(())
}

fn draw_title_tabs(
    frame: &mut Frame,
    state: &mut State,
    area: ratatui::layout::Rect,
) -> Result<()> {
    let title_length =
        u16::try_from(crate::APP_TITLE_FULL.len()).context("title length greater than 16 bits")?;
    let layout = Layout::new(
        Direction::Horizontal,
        [Constraint::Length(title_length + 4), Constraint::Fill(1)],
    )
    .split(area.inner(ratatui::layout::Margin::new(1, 0)));
    let title_area = *layout.first().context("ui index")?;
    let tabs_area = *layout.get(1).context("ui index")?;
    frame.render_widget(
        Block::new()
            .title(crate::APP_TITLE_FULL)
            .title_alignment(ratatui::layout::Alignment::Center)
            .fg(state.config.ui.colors.text.title)
            .bold(),
        title_area,
    );
    let selected_tab_index = match state.ui.focus.tab {
        crate::app::focus::Tab::Conversation => 0,
        crate::app::focus::Tab::Config => 1,
        crate::app::focus::Tab::Debug => 2,
    };
    let tabs_widget = ratatui::widgets::Tabs::new(vec!["Chat", "Config", "Debug"])
        .style(ratatui::style::Style::default().fg(state.config.ui.colors.frame.inactive))
        .highlight_style(
            ratatui::style::Style::default()
                .fg(state.config.ui.colors.frame.normal)
                .bold(),
        )
        .divider(ratatui::symbols::DOT.fg(state.config.ui.colors.text.inactive))
        .select(selected_tab_index);
    let tabs_block = ratatui::widgets::Block::new()
        .borders(ratatui::widgets::Borders::LEFT)
        .fg(state.config.ui.colors.frame.inactive);
    frame.render_widget(&tabs_block, tabs_area);
    frame.render_widget(tabs_widget, tabs_block.inner(tabs_area));
    Ok(())
}
