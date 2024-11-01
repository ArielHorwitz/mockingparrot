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
            chat::draw(frame, main_layout, state, conversation_scope).context("draw chat tab")?;
        }
        Scope::Config(config_scope) => {
            config::draw(frame, main_layout, state, config_scope).context("draw config")?;
        }
        Scope::Debug => debug::draw(frame, main_layout, state),
    };
    Ok(())
}
