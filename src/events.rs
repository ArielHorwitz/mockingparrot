use crate::api::GptMessage;
use crate::state::State;
use crate::ui::{UiState, ViewTab};
use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::process::Command;

const API_ERROR_FEEDBACK: &str = "An error occured, see debug logs.";
const API_ERROR_SYSTEM_MESSAGE: &str = "Failed to get a response from the assistant.";
const MESSAGE_FILE: &str = ".local/share/hummingparrot/message_text";

pub enum HandleEventResult {
    None,
    Redraw,
    Quit,
}

pub async fn handle_events(
    timeout: u64,
    state: &mut State,
    ui_state: &mut UiState<'_>,
) -> Result<HandleEventResult> {
    if !event::poll(std::time::Duration::from_millis(timeout)).context("poll terminal events")? {
        return Ok(HandleEventResult::None);
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
    Ok(HandleEventResult::None)
}

async fn handle_keys(
    key_event: KeyEvent,
    state: &mut State,
    ui_state: &mut UiState<'_>,
) -> Result<HandleEventResult> {
    if key_event.kind != KeyEventKind::Press {
        return Ok(HandleEventResult::None);
    }
    ui_state.key_event_debug = format!("{:?} {:?}", key_event.modifiers, key_event.code);
    match (key_event.code, key_event.modifiers) {
        (KeyCode::Char('q'), KeyModifiers::CONTROL) => return Ok(HandleEventResult::Quit),
        (KeyCode::BackTab, KeyModifiers::SHIFT) => ui_state.tab = ui_state.tab.next_tab(),
        (KeyCode::F(1), _) => ui_state.tab = ViewTab::Conversation,
        (KeyCode::F(2), _) => ui_state.tab = ViewTab::Config,
        _ => {
            return match ui_state.tab {
                ViewTab::Conversation => handle_conversation_keys(key_event, state, ui_state)
                    .await
                    .context("handle conversation keys"),
                ViewTab::Config => handle_config_keys(key_event, state, ui_state)
                    .await
                    .context("handle config keys"),
            };
        }
    };
    Ok(HandleEventResult::None)
}

async fn handle_config_keys(
    key_event: KeyEvent,
    state: &mut State,
    ui_state: &mut UiState<'_>,
) -> Result<HandleEventResult> {
    match (key_event.code, key_event.modifiers) {
        (KeyCode::Char('d'), KeyModifiers::NONE) => ui_state.debug = !ui_state.debug,
        (KeyCode::Char('t'), KeyModifiers::NONE) => state.config.chat.temperature += 0.05,
        (KeyCode::Char('T'), KeyModifiers::SHIFT) => state.config.chat.temperature -= 0.05,
        (KeyCode::Char('p'), KeyModifiers::NONE) => state.config.chat.top_p += 0.05,
        (KeyCode::Char('P'), KeyModifiers::SHIFT) => state.config.chat.top_p -= 0.05,
        (KeyCode::Char('f'), KeyModifiers::NONE) => state.config.chat.frequency_penalty += 0.05,
        (KeyCode::Char('F'), KeyModifiers::SHIFT) => state.config.chat.frequency_penalty -= 0.05,
        (KeyCode::Char('r'), KeyModifiers::NONE) => state.config.chat.presence_penalty += 0.05,
        (KeyCode::Char('R'), KeyModifiers::SHIFT) => state.config.chat.presence_penalty -= 0.05,
        _ => (),
    };
    Ok(HandleEventResult::None)
}

async fn handle_conversation_keys(
    key_event: KeyEvent,
    state: &mut State,
    ui_state: &mut UiState<'_>,
) -> Result<HandleEventResult> {
    match (key_event.code, key_event.modifiers) {
        (KeyCode::Enter, KeyModifiers::ALT) => {
            let message = GptMessage::new_user_message(ui_state.textarea.lines().join("\n"));
            state.conversation.add_message(message);
            do_prompt(state, ui_state).await?;
        }
        (KeyCode::Char('e'), KeyModifiers::ALT) => {
            let message_text = get_message_text_from_editor(&state.config)
                .context("get message text from editor")?;
            ui_state.textarea.select_all();
            ui_state.textarea.cut();
            ui_state.textarea.insert_str(&message_text);
            return Ok(HandleEventResult::Redraw);
        }
        _ => {
            ui_state.textarea.input(key_event);
        }
    };
    Ok(HandleEventResult::None)
}

fn get_message_text_from_editor(config: &crate::config::Config) -> Result<String> {
    let user_home_dir = std::env::var("HOME").context("get HOME environment variable")?;
    let message_file = std::path::Path::new(&user_home_dir).join(MESSAGE_FILE);
    let message_dir = message_file
        .parent()
        .expect("get message file parent directory");
    std::fs::create_dir_all(message_dir).context("create directory for message file")?;
    let mut editor_command_iter = config.ui.editor_command.iter();
    let editor_process_output =
        Command::new(editor_command_iter.next().context("editor command empty")?)
            .args(editor_command_iter.collect::<Vec<&String>>())
            .arg(&message_file)
            .output()
            .context("run editor")?;
    if !editor_process_output.status.success() {
        anyhow::bail!(format!(
            "editor process failed: {}",
            editor_process_output.status
        ));
    }
    let message_text = std::fs::read_to_string(message_file)
        .context("read message text from file")?
        .trim()
        .to_owned();
    Ok(message_text)
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
