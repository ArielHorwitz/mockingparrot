use crate::api::GptMessage;
use crate::state::{Conversation, State, ViewTab};
use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

mod actions;

use actions::{do_prompt, get_message_text_from_editor};

pub enum HandleEventResult {
    None,
    Redraw,
    Quit,
}

pub async fn handle(timeout: u64, state: &mut State) -> Result<HandleEventResult> {
    if !event::poll(std::time::Duration::from_millis(timeout)).context("poll terminal events")? {
        return Ok(HandleEventResult::None);
    };
    let terminal_event = event::read().context("read terminal event")?;
    match terminal_event {
        Event::Key(key_event) => return handle_keys(key_event, state).await,
        Event::FocusGained => state.add_debug_log("focus gained"),
        Event::FocusLost => state.add_debug_log("focus lost"),
        Event::Mouse(ev) => state.add_debug_log(format!("mouse {ev:#?}")),
        Event::Paste(p) => state.add_debug_log(format!("paste {p:#?}")),
        Event::Resize(x, y) => state.add_debug_log(format!("resize {x}x{y}")),
    };
    Ok(HandleEventResult::None)
}

async fn handle_keys(key_event: KeyEvent, state: &mut State) -> Result<HandleEventResult> {
    if key_event.kind != KeyEventKind::Press {
        return Ok(HandleEventResult::None);
    }
    match (key_event.code, key_event.modifiers) {
        (KeyCode::Char('q'), KeyModifiers::CONTROL) => return Ok(HandleEventResult::Quit),
        (KeyCode::BackTab, KeyModifiers::SHIFT) => state.tab = state.tab.next_tab(),
        (KeyCode::F(1), _) => state.tab = ViewTab::Conversation,
        (KeyCode::F(2), _) => state.tab = ViewTab::Config,
        _ => {
            match state.tab {
                ViewTab::Conversation => {
                    return handle_conversation_keys(key_event, state)
                        .await
                        .context("handle conversation keys")
                }
                ViewTab::NewConversation => handle_new_conversation_keys(key_event, state),
                ViewTab::Config => handle_config_keys(key_event, state),
            };
        }
    };
    Ok(HandleEventResult::None)
}

fn handle_config_keys(key_event: KeyEvent, state: &mut State) -> HandleEventResult {
    match (key_event.code, key_event.modifiers) {
        (KeyCode::PageUp, KeyModifiers::NONE) => {
            state.debug_logs_scroll = state.debug_logs_scroll.saturating_sub(1);
        }
        (KeyCode::PageDown, KeyModifiers::NONE) => {
            state.debug_logs_scroll = state.debug_logs_scroll.saturating_add(1);
        }
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
    HandleEventResult::None
}

fn handle_new_conversation_keys(key_event: KeyEvent, state: &mut State) -> HandleEventResult {
    match (key_event.code, key_event.modifiers) {
        (KeyCode::Esc, KeyModifiers::NONE) => state.tab = ViewTab::Conversation,
        (KeyCode::Enter, KeyModifiers::NONE) => {
            if let Some(instruction_index) = state.system_instruction_selection.selected() {
                if let Some(system_instructions) =
                    state.config.system.instructions.get(instruction_index)
                {
                    state.conversation = Conversation::new(system_instructions.message.clone());
                };
            }
            state.tab = ViewTab::Conversation;
        }
        (KeyCode::Down, KeyModifiers::NONE) => {
            let mut new_selection = state.system_instruction_selection.selected().unwrap_or(0) + 1;
            if new_selection >= state.config.system.instructions.len() {
                new_selection = 0;
            }
            state
                .system_instruction_selection
                .select(Some(new_selection));
            state.add_debug_log(format!(
                "selected preset {:?}",
                state.system_instruction_selection.selected()
            ));
        }
        (KeyCode::Up, KeyModifiers::NONE) => {
            let new_selection = state
                .system_instruction_selection
                .selected()
                .unwrap_or(0)
                .checked_sub(1)
                .unwrap_or(state.config.system.instructions.len() - 1);
            state
                .system_instruction_selection
                .select(Some(new_selection));
            state.add_debug_log(format!(
                "selected preset {:?}",
                state.system_instruction_selection.selected()
            ));
        }
        _ => (),
    };
    HandleEventResult::None
}

async fn handle_conversation_keys(
    key_event: KeyEvent,
    state: &mut State,
) -> Result<HandleEventResult> {
    match (key_event.code, key_event.modifiers) {
        (KeyCode::PageUp, KeyModifiers::NONE) => {
            state.conversation_scroll = state.conversation_scroll.saturating_sub(1);
        }
        (KeyCode::PageDown, KeyModifiers::NONE) => {
            state.conversation_scroll = state.conversation_scroll.saturating_add(1);
        }
        (KeyCode::Char('n'), KeyModifiers::CONTROL) => state.tab = ViewTab::NewConversation,
        (KeyCode::Enter, KeyModifiers::ALT) => {
            let message = GptMessage::new_user_message(state.prompt_textarea.lines().join("\n"));
            state.conversation.add_message(message);
            do_prompt(state).await?;
        }
        (KeyCode::Char('e'), KeyModifiers::ALT) => {
            let initial_text = state.prompt_textarea.lines().join("\n");
            let message_text = get_message_text_from_editor(&state.config, initial_text.as_str())
                .context("get message text from editor")?;
            state.prompt_textarea.select_all();
            state.prompt_textarea.cut();
            state.prompt_textarea.insert_str(&message_text);
            return Ok(HandleEventResult::Redraw);
        }
        _ => {
            state.prompt_textarea.input(key_event);
        }
    };
    Ok(HandleEventResult::None)
}
