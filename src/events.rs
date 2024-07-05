use crate::api::GptMessage;
use crate::hotkeys::{HotkeyAction, HotkeyMap};
use crate::state::{Conversation, State, ViewTab};
use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyEvent, KeyEventKind};

mod actions;

use actions::{do_prompt, get_message_text_from_editor};

pub enum HandleEventResult {
    None,
    Redraw,
    Quit,
}

pub async fn handle(
    timeout: u64,
    state: &mut State,
    hotkey_map: &HotkeyMap,
) -> Result<HandleEventResult> {
    if !event::poll(std::time::Duration::from_millis(timeout)).context("poll terminal events")? {
        return Ok(HandleEventResult::None);
    };
    let terminal_event = event::read().context("read terminal event")?;
    match terminal_event {
        Event::Key(key_event) => return handle_keys(key_event, state, hotkey_map).await,
        Event::FocusGained => state.add_debug_log("focus gained"),
        Event::FocusLost => state.add_debug_log("focus lost"),
        Event::Mouse(ev) => state.add_debug_log(format!("mouse {ev:#?}")),
        Event::Paste(p) => state.add_debug_log(format!("paste {p:#?}")),
        Event::Resize(x, y) => state.add_debug_log(format!("resize {x}x{y}")),
    };
    Ok(HandleEventResult::None)
}

async fn handle_keys(
    key_event: KeyEvent,
    state: &mut State,
    hotkey_map: &HotkeyMap,
) -> Result<HandleEventResult> {
    if key_event.kind != KeyEventKind::Press {
        return Ok(HandleEventResult::None);
    }
    if let Some(hotkey_action) = hotkey_map.get(&(key_event)) {
        match hotkey_action {
            HotkeyAction::QuitProgram => return Ok(HandleEventResult::Quit),
            HotkeyAction::NextTab => state.tab = state.tab.next_tab(),
            HotkeyAction::ViewConversationTab => state.tab = ViewTab::Conversation,
            HotkeyAction::ViewConfigTab => state.tab = ViewTab::Config,
            _ => {
                match state.tab {
                    ViewTab::Conversation => {
                        return handle_conversation_keys(key_event, state, hotkey_map)
                            .await
                            .context("handle conversation keys")
                    }
                    ViewTab::NewConversation => {
                        handle_new_conversation_keys(key_event, state, hotkey_map)
                    }
                    ViewTab::Config => handle_config_keys(key_event, state, hotkey_map),
                };
            }
        }
    } else {
        match state.tab {
            ViewTab::Conversation => {
                return handle_conversation_keys(key_event, state, hotkey_map)
                    .await
                    .context("handle conversation keys")
            }
            ViewTab::NewConversation => handle_new_conversation_keys(key_event, state, hotkey_map),
            ViewTab::Config => handle_config_keys(key_event, state, hotkey_map),
        };
    }
    Ok(HandleEventResult::None)
}

fn handle_config_keys(
    key_event: KeyEvent,
    state: &mut State,
    hotkey_map: &HotkeyMap,
) -> HandleEventResult {
    if let Some(hotkey_action) = hotkey_map.get(&(key_event)) {
        match hotkey_action {
            HotkeyAction::DebugLogsScrollUp => {
                state.debug_logs_scroll = state.debug_logs_scroll.saturating_sub(1);
            }
            HotkeyAction::DebugLogsScrollDown => {
                state.debug_logs_scroll = state.debug_logs_scroll.saturating_add(1);
            }
            HotkeyAction::IncrementTempurature => state.config.chat.temperature += 0.05,
            HotkeyAction::DecrementTempurature => state.config.chat.temperature -= 0.05,
            HotkeyAction::IncrementTopP => state.config.chat.top_p += 0.05,
            HotkeyAction::DecrementTopP => state.config.chat.top_p -= 0.05,
            HotkeyAction::IncrementFrequencyPenalty => state.config.chat.frequency_penalty += 0.05,
            HotkeyAction::DecrementFrequencyPenalty => state.config.chat.frequency_penalty -= 0.05,
            HotkeyAction::IncrementPresencePenalty => state.config.chat.presence_penalty += 0.05,
            HotkeyAction::DecrementPresencePenalty => state.config.chat.presence_penalty -= 0.05,
            _ => (),
        };
    };
    HandleEventResult::None
}

fn handle_new_conversation_keys(
    key_event: KeyEvent,
    state: &mut State,
    hotkey_map: &HotkeyMap,
) -> HandleEventResult {
    if let Some(hotkey_action) = hotkey_map.get(&(key_event)) {
        match hotkey_action {
            HotkeyAction::EscNewConversation => state.tab = ViewTab::Conversation,
            HotkeyAction::EnterNewConversation => {
                if let Some(system_instructions) = state
                    .config
                    .system
                    .instructions
                    .get(state.system_instruction_selection)
                {
                    state.conversation = Conversation::new(system_instructions.message.clone());
                };
                state.tab = ViewTab::Conversation;
            }
            HotkeyAction::DownNewConversation => {
                let new_selection = state.system_instruction_selection.saturating_add(1);
                if new_selection >= state.config.system.instructions.len() {
                    state.system_instruction_selection = 0;
                } else {
                    state.system_instruction_selection = new_selection;
                }
            }
            HotkeyAction::UpNewConversation => {
                let new_selection = state
                    .system_instruction_selection
                    .checked_sub(1)
                    .unwrap_or(state.config.system.instructions.len() - 1);
                state.system_instruction_selection = new_selection;
            }
            _ => (),
        }
    };
    HandleEventResult::None
}

async fn handle_conversation_keys(
    key_event: KeyEvent,
    state: &mut State,
    hotkey_map: &HotkeyMap,
) -> Result<HandleEventResult> {
    if let Some(hotkey_action) = hotkey_map.get(&(key_event)) {
        match hotkey_action {
            HotkeyAction::ConversationScrollUp => {
                state.conversation_scroll = state.conversation_scroll.saturating_sub(1);
            }
            HotkeyAction::ConversationScrollDown => {
                state.conversation_scroll = state.conversation_scroll.saturating_add(1);
            }
            HotkeyAction::NewConversation => state.tab = ViewTab::NewConversation,
            HotkeyAction::SendPrompt => {
                let text = state.prompt_textarea.lines().join("\n");
                if text.trim().is_empty() {
                    state.set_status_bar_text("Cannot send empty message.");
                    return Ok(HandleEventResult::None);
                }
                let message = GptMessage::new_user_message(text);
                state.conversation.add_message(message);
                do_prompt(state).await?;
            }
            HotkeyAction::GetMessageFromEditor => {
                let initial_text = state.prompt_textarea.lines().join("\n");
                let message_text =
                    get_message_text_from_editor(&state.config, initial_text.as_str())
                        .context("get message text from editor")?;
                state.prompt_textarea.select_all();
                state.prompt_textarea.cut();
                state.prompt_textarea.insert_str(&message_text);
                return Ok(HandleEventResult::Redraw);
            }
            _ => {
                state.prompt_textarea.input(key_event);
            }
        }
    } else {
        state.prompt_textarea.input(key_event);
    }
    Ok(HandleEventResult::None)
}
