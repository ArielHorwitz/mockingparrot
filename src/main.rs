use anyhow::{Context, Result};
use crossterm::ExecutableCommand;
use ratatui::prelude::{Backend, CrosstermBackend, Terminal};
use std::{io::stdout, path::Path};

mod api;
mod config;
mod events;
mod state;
mod ui;

use api::GptMessage;
use config::Config;
use events::EventResult;
use state::State;
use ui::UiState;

const FRAME_DURATION_MS: u64 = 50;
const APP_TITLE: &str = "HummingParrot";
const APP_TITLE_FULL: &str = "HummingParrot AI Chat Client";
const API_ERROR_FEEDBACK: &str = "An error occured, see debug logs.";
const API_ERROR_SYSTEM_MESSAGE: &str = "Failed to get a response from the assistant.";

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
    let mut state = State::new(config.clone());
    let mut ui_state = ui::UiState::default();
    loop {
        terminal
            .draw(|frame| ui::draw_ui_frame(frame, &state, &ui_state))
            .context("draw frame")?;
        match events::handle_events(FRAME_DURATION_MS, &mut state, &mut ui_state).context("handle events")? {
            EventResult::None => (),
            EventResult::Prompt => {
                let message = GptMessage::new_user_message(ui_state.textarea.lines().join("\n"));
                state.conversation.add_message(message);
                do_prompt(&mut state, &mut ui_state).await?;
            }
            EventResult::Feedback(text) => ui_state.status_bar_text = text,
            EventResult::Quit => return Ok(()),
        };
    }
}

async fn do_prompt(state: &mut State, ui_state: &mut UiState<'_>) -> Result<()> {
    let raw_response = api::call_api(&reqwest::Client::new(), &state.config, &state.conversation)
        .await
        .context("decode response");
    if let Ok(response) = raw_response {
        let message = &response
            .choices
            .first()
            .context("missing response choices")?
            .message;
        state.conversation.add_message(message.clone());
        ui_state.status_bar_text = format!("AI responded. {}", response.usage);
    } else {
        API_ERROR_FEEDBACK.clone_into(&mut ui_state.status_bar_text);
        state.conversation.add_message(GptMessage::new_system_message(API_ERROR_SYSTEM_MESSAGE.to_owned()));
        ui_state.feedback = format!("{raw_response:#?}");
    }
    Ok(())
}
