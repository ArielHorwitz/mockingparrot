use crate::app::focus::Config as ConfigFocus;
use crate::app::state::State;
use anyhow::{Context, Result};
use ratatui::{
    prelude::{Constraint, Direction, Layout, Rect, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw(
    frame: &mut Frame,
    rect: Rect,
    state: &mut State,
    config_scope: ConfigFocus,
) -> Result<()> {
    let outer_layout = Layout::new(
        Direction::Vertical,
        [Constraint::Length(2), Constraint::Fill(1)],
    )
    .split(rect);
    let top_layout = outer_layout.first().context("ui index")?;
    let bottom_layout = outer_layout.get(1).context("ui index")?;

    let config_block = Block::new()
        .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
        .border_style(state.config.ui.colors.frame.normal)
        .title("Configuration")
        .title_style(state.config.ui.colors.frame.title);

    let text_style = Style::new().fg(state.config.ui.colors.text.normal);

    frame.render_widget(&config_block, *top_layout);
    frame.render_widget(
        Paragraph::new(format!(
            "Config file: {}",
            state.paths.get_config_file().display()
        ))
        .style(text_style),
        config_block.inner(*top_layout),
    );
    let (title, config_details) = match config_scope {
        ConfigFocus::OpenAi => ("OpenAI", format!("{:#?}", state.models.openai)),
        ConfigFocus::Anthropic => ("Anthropic", format!("{:#?}", state.models.anthropic)),
    };

    let config_block = Block::new()
        .borders(Borders::ALL)
        .border_style(state.config.ui.colors.frame.normal)
        .title(format!("{title} Configuration"))
        .title_style(state.config.ui.colors.frame.title);
    frame.render_widget(&config_block, *bottom_layout);
    frame.render_widget(
        Paragraph::new(config_details).style(text_style),
        config_block.inner(*bottom_layout),
    );
    Ok(())
}
