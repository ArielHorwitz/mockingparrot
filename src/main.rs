use anyhow::{Context, Result};
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::{Backend, CrosstermBackend, Terminal};
use std::{io::stdout, time::Instant};

mod api;
mod config;
mod events;
mod state;
mod ui;

use config::Config;
use events::EventResult;
use state::State;
use ui::draw_ui_frame;

const FRAME_DURATION_MS: u64 = 250;

fn main() {
    if let Err(e) = run() {
        eprintln!("{e:?}");
    }
}

fn run() -> Result<()> {
    let mut config = Config::default();
    include_str!("../APIKEY").trim().clone_into(&mut config.api.key);
    println!("{config:?}");
    // Setup terminal
    stdout()
        .execute(EnterAlternateScreen)
        .context("enter alternate terminal screen mode")?;
    enable_raw_mode().context("enable raw mode")?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))
        .context("new terminal with crossterm backend")?;
    terminal.clear().context("clear terminal")?;
    // Run app
    let app_result = run_app(&mut terminal, config).context("app error");
    // Clean up terminal
    stdout()
        .execute(LeaveAlternateScreen)
        .context("leave alternate terminal screen mode")?;
    disable_raw_mode().context("disabled raw mode")?;
    // Return app run result
    app_result
}

pub fn run_app(terminal: &mut Terminal<impl Backend>, config: Config) -> Result<()> {
    let mut state = State::default();
    let mut textarea = tui_textarea::TextArea::default();
    loop {
        state.frame_time = state.last_frame.elapsed();
        state.last_frame = Instant::now();
        state.frame_count += 1;
        terminal
            .draw(|frame| draw_ui_frame(frame, &state, &textarea))
            .context("draw frame")?;
        match events::handle_events(FRAME_DURATION_MS, &mut textarea).context("handle events")? {
            EventResult::None => (),
            EventResult::Prompt => {
                state.feedback = do_prompt(&config, textarea.lines().join("\n"))?
            }
            EventResult::Feedback(text) => state.feedback = text,
            EventResult::Quit => return Ok(()),
        };
    }
}

fn do_prompt(config: &Config, prompt: String) -> Result<String> {
    let rt = tokio::runtime::Runtime::new()?;
    let response = rt.block_on(api::call_api(&reqwest::Client::new(), config, prompt.as_str()))?;
    let text = rt.block_on(response.text())?;
    Ok(format!("\n{text}"))
}
