use anyhow::{Context, Result};
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::{Backend, CrosstermBackend, Terminal};
use std::{io::stdout, path::Path};

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
const APP_TITLE: &str = "HummingParrot";
const APP_TITLE_FULL: &str = "HummingParrot AI Chat Client";

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
        .execute(EnterAlternateScreen)
        .context("enter alternate terminal screen mode")?;
    enable_raw_mode().context("enable raw mode")?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))
        .context("new terminal with crossterm backend")?;
    terminal.clear().context("clear terminal")?;
    // Run app
    let app_result = run_app(&mut terminal, config).await;
    // Clean up terminal
    stdout()
        .execute(LeaveAlternateScreen)
        .context("leave alternate terminal screen mode")?;
    disable_raw_mode().context("disabled raw mode")?;
    // Return app run result
    app_result
}

pub async fn run_app(terminal: &mut Terminal<impl Backend>, config: Config) -> Result<()> {
    let mut state = State::new(config.clone());
    let mut textarea = tui_textarea::TextArea::default();
    let mut frame_count = 0;
    loop {
        frame_count += 1;
        terminal
            .draw(|frame| draw_ui_frame(frame, &state, &textarea, frame_count))
            .context("draw frame")?;
        match events::handle_events(FRAME_DURATION_MS, &mut textarea).context("handle events")? {
            EventResult::None => (),
            EventResult::Prompt => {
                state
                    .conversation
                    .add_message(crate::api::GptMessage::new_user_message(
                        textarea.lines().join("\n"),
                    ));
                do_prompt(&config, &mut state).await?;
            }
            EventResult::Feedback(text) => state.status_bar_text = text,
            EventResult::Quit => return Ok(()),
        };
    }
}

async fn do_prompt(config: &Config, state: &mut State) -> Result<()> {
    let response = api::call_api(&reqwest::Client::new(), config, &state.conversation)
        .await
        .context("decode response (misconfigured API key?)")?;
    let response_message = &response
        .choices
        .first()
        .context("missing response choices")?
        .message;
    state.conversation.add_message(response_message.clone());
    state.status_bar_text = format!("AI responded. {}", &response.usage);
    Ok(())
}
