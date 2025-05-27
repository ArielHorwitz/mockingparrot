use crate::app::state::State;
use anyhow::{Context, Result};
use ratatui::{
    prelude::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw(frame: &mut Frame, rect: Rect, state: &mut State) -> Result<()> {
    let outer_layout = Layout::new(
        Direction::Vertical,
        [Constraint::Length(3), Constraint::Fill(1)],
    )
    .split(rect);
    let filepath_layout = outer_layout.first().context("ui index")?;
    let display_layout = outer_layout.get(1).context("ui index")?;

    let config_block = Block::new()
        .borders(Borders::ALL)
        .border_style(state.theme.frame(true))
        .title("Configuration file")
        .title_style(state.theme.title());

    let text_style = state.theme.text(true);

    frame.render_widget(&config_block, *filepath_layout);
    frame.render_widget(
        Paragraph::new(format!(
            "Config file: {}",
            state.paths.get_config_file().display(),
        ))
        .style(text_style),
        config_block.inner(*filepath_layout),
    );

    let config_block = Block::new()
        .borders(Borders::ALL)
        .border_style(state.theme.frame(true))
        .title("Configuration")
        .title_style(state.theme.title());
    frame.render_widget(&config_block, *display_layout);
    frame.render_widget(
        Paragraph::new(format!("{:#?}", state.config)).style(text_style),
        config_block.inner(*display_layout),
    );

    Ok(())
}
