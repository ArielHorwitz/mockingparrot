use anyhow::{Context, Result};
use crossterm::ExecutableCommand;
use ratatui::prelude::{Backend, CrosstermBackend, Terminal};
use std::{io::stdout, path::Path};

use hummingparrot::config::Config;
use hummingparrot::events;
use hummingparrot::state::State;
use hummingparrot::ui;

const FRAME_DURATION_MS: u64 = 50;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{e:?}");
    }
}

async fn run() -> Result<()> {
    let config_toml =
        std::fs::read_to_string(Path::new("config.toml")).context("read config file")?;
    let config = toml::from_str(&config_toml).context("parse config file toml")?;
    // Setup terminal
    stdout()
        .execute(crossterm::terminal::EnterAlternateScreen)
        .context("enter alternate terminal screen mode")?;
    crossterm::terminal::enable_raw_mode().context("enable raw mode")?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))
        .context("new terminal with crossterm backend")?;
    terminal.clear().context("clear terminal")?;
    // Run app
    let app_result = run_app(&mut terminal, config).await;
    // Clean up terminal
    stdout()
        .execute(crossterm::terminal::LeaveAlternateScreen)
        .context("leave alternate terminal screen mode")?;
    crossterm::terminal::disable_raw_mode().context("disabled raw mode")?;
    // Return app run result
    app_result
}

pub async fn run_app(terminal: &mut Terminal<impl Backend>, config: Config) -> Result<()> {
    let mut state = State::from_config(config.clone()).context("new app state")?;
    loop {
        terminal
            .draw(|frame| {
                if let Err(e) = ui::draw_ui_frame(frame, &state) {
                    todo!("log error: {e}");
                }
            })
            .context("draw frame")?;
        match events::handle_events(FRAME_DURATION_MS, &mut state)
            .await
            .context("handle events")?
        {
            events::HandleEventResult::None => (),
            events::HandleEventResult::Redraw => terminal.clear().context("clear terminal")?,
            events::HandleEventResult::Quit => return Ok(()),
        };
    }
}
