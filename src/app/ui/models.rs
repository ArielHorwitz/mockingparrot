use crate::app::state::{SelectedModel, State};
use anyhow::{Context, Result};
use ratatui::{
    prelude::{Constraint, Direction, Layout, Rect},
    text::{Line, Text},
    widgets::{Block, Borders},
    Frame,
};

pub fn draw(frame: &mut Frame, rect: Rect, state: &mut State) -> Result<()> {
    let outer_layout = Layout::new(
        Direction::Horizontal,
        [Constraint::Fill(1), Constraint::Fill(1)],
    )
    .split(rect);
    let openai_layout = outer_layout.first().context("ui index")?;
    let anthropic_layout = outer_layout.get(1).context("ui index")?;

    let openai_names: Vec<&str> = state
        .models
        .openai
        .iter()
        .map(|m| m.name.as_str())
        .collect();
    let anthropic_names: Vec<&str> = state
        .models
        .anthropic
        .iter()
        .map(|m| m.name.as_str())
        .collect();
    let (openai_selection, anthropic_selection) = match state.ui.selected_model {
        SelectedModel::OpenAi(index) => (Some(index), None),
        SelectedModel::Anthropic(index) => (None, Some(index)),
    };
    for (title, layout, models, selection) in [
        ("OpenAI", openai_layout, openai_names, openai_selection),
        (
            "Anthropic",
            anthropic_layout,
            anthropic_names,
            anthropic_selection,
        ),
    ] {
        let config_block = Block::new()
            .borders(Borders::ALL)
            .border_style(state.theme.frame(true))
            .title(format!("{title} models"))
            .title_style(state.theme.title());
        frame.render_widget(&config_block, *layout);
        let mut lines = Vec::new();
        for (index, model) in models.iter().enumerate() {
            let style = if Some(index) == selection {
                state.theme.name(true)
            } else {
                state.theme.text(true)
            };
            lines.push(Line::styled(*model, style));
        }
        frame.render_widget(Text::from_iter(lines), config_block.inner(*layout));
    }

    Ok(())
}
