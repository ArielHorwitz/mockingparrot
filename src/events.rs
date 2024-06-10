use crate::api::GptMessage;
use crate::state::State;
use crate::ui::{UiState, ViewTab};
use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

const API_ERROR_FEEDBACK: &str = "An error occured, see debug logs.";
const API_ERROR_SYSTEM_MESSAGE: &str = "Failed to get a response from the assistant.";

pub async fn handle_events(
    timeout: u64,
    state: &mut State,
    ui_state: &mut UiState<'_>,
) -> Result<bool> {
    if !event::poll(std::time::Duration::from_millis(timeout)).context("poll terminal events")? {
        return Ok(false);
    };
    let terminal_event = event::read().context("read terminal event")?;
    match terminal_event {
        Event::Key(key_event) => return handle_keys(key_event, state, ui_state).await,
        Event::FocusGained => ui_state.status_bar_text = String::from("focus gained"),
        Event::FocusLost => ui_state.status_bar_text = String::from("focus lost"),
        Event::Mouse(ev) => ui_state.status_bar_text = format!("mouse {ev:#?}"),
        Event::Paste(p) => ui_state.status_bar_text = format!("paste {p:#?}"),
        Event::Resize(x, y) => ui_state.status_bar_text = format!("resize {x}x{y}"),
    };
    Ok(false)
}

async fn handle_keys(
    key_event: KeyEvent,
    state: &mut State,
    ui_state: &mut UiState<'_>,
) -> Result<bool> {
    if key_event.kind != KeyEventKind::Press {
        return Ok(false);
    }
    ui_state.key_event_debug = format!("{:?} {:?}", key_event.modifiers, key_event.code);
    match (key_event.code, key_event.modifiers) {
        (KeyCode::Char('q'), KeyModifiers::CONTROL) => return Ok(true),
        (KeyCode::BackTab, KeyModifiers::SHIFT) => ui_state.tab = ui_state.tab.next_tab(),
        (KeyCode::F(1), _) => ui_state.tab = ViewTab::Conversation,
        (KeyCode::F(2), _) => ui_state.tab = ViewTab::Config,
        _ => {
            if ui_state.tab == ViewTab::Conversation {
                match (key_event.code, key_event.modifiers) {
                    (KeyCode::Enter, KeyModifiers::ALT) => {
                        let message =
                            GptMessage::new_user_message(ui_state.textarea.lines().join("\n"));
                        state.conversation.add_message(message);
                        do_prompt(state, ui_state).await?;
                    }
                    _ => {
                        ui_state.textarea.input(key_event);
                    }
                };
            }
            if ui_state.tab == ViewTab::Config {
                match (key_event.code, key_event.modifiers) {
                    (KeyCode::Char('d'), KeyModifiers::NONE) => ui_state.debug = !ui_state.debug,
                    (KeyCode::Char('t'), KeyModifiers::NONE) => {
                        state.config.chat.temperature += 0.05
                    }
                    (KeyCode::Char('T'), KeyModifiers::SHIFT) => {
                        state.config.chat.temperature -= 0.05
                    }
                    (KeyCode::Char('p'), KeyModifiers::NONE) => state.config.chat.top_p += 0.05,
                    (KeyCode::Char('P'), KeyModifiers::SHIFT) => state.config.chat.top_p -= 0.05,
                    (KeyCode::Char('f'), KeyModifiers::NONE) => {
                        state.config.chat.frequency_penalty += 0.05
                    }
                    (KeyCode::Char('F'), KeyModifiers::SHIFT) => {
                        state.config.chat.frequency_penalty -= 0.05
                    }
                    (KeyCode::Char('r'), KeyModifiers::NONE) => {
                        state.config.chat.presence_penalty += 0.05
                    }
                    (KeyCode::Char('R'), KeyModifiers::SHIFT) => {
                        state.config.chat.presence_penalty -= 0.05
                    }
                    _ => (),
                };
            }
        }
    };
    Ok(false)
}

async fn do_prompt(state: &mut State, ui_state: &mut UiState<'_>) -> Result<()> {
    let raw_response =
        crate::api::call_api(&reqwest::Client::new(), &state.config, &state.conversation)
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
        state
            .conversation
            .add_message(GptMessage::new_system_message(
                API_ERROR_SYSTEM_MESSAGE.to_owned(),
            ));
        ui_state.feedback = format!("{raw_response:#?}");
    }
    Ok(())
}
