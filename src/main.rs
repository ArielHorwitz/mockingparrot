use anyhow::{Context, Result};
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::{Backend, CrosstermBackend, Terminal};
use std::{io::stdout, time::Instant};

mod events;
mod state;
mod ui;

use events::EventResult;
use state::State;
use ui::draw_frame;

const FRAME_DURATION_MS: u64 = 250;

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
    }
}

fn run() -> Result<()> {
    // Setup terminal
    stdout()
        .execute(EnterAlternateScreen)
        .context("enter alternate terminal screen mode")?;
    enable_raw_mode().context("enable raw mode")?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))
        .context("new terminal with crossterm backend")?;
    terminal.clear().context("clear terminal")?;
    // Run app
    let app_result = run_app(&mut terminal);
    // Clean up terminal
    stdout()
        .execute(LeaveAlternateScreen)
        .context("leave alternate terminal screen mode")?;
    disable_raw_mode().context("disabled raw mode")?;
    // Return app run result
    app_result
}

pub fn run_app(terminal: &mut Terminal<impl Backend>) -> Result<()> {
    let mut state = State::default();
    loop {
        state.frame_time = state.last_frame.elapsed();
        state.last_frame = Instant::now();
        state.frame_count += 1;
        terminal.draw(|frame| draw_frame(frame, &mut state)).context("draw frame")?;
        match events::handle_events(FRAME_DURATION_MS).context("handle events")? {
            EventResult::None => (),
            EventResult::Feedback(text) => state.feedback = text,
            EventResult::Quit => return Ok(()),
        };
    }
}
