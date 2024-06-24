use std::collections::HashMap;

use crate::api::GptMessage;
use crate::state::{Conversation, State, ViewTab};
use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

mod actions;

use actions::{do_prompt, get_message_text_from_editor};

#[derive(Debug)]
pub enum HotkeyAction {
    NextTab,
    IncrementTempurature,
    DecrementTempurature,
    IncrementTopP,
    DecrementTopP,
    IncrementFrequencyPenalty,
    DecrementFrequencyPenalty,
    IncrementPresencePenalty,
    DecrementPresencePenalty,
    SendPrompt,
    GetMessageFromEditor,
    ViewConfigTab,
    ViewConversationTab,
    QuitProgram,
    DebugLogsScrollUp,
    DebugLogsScrollDown,
    ConversationScrollUp,
    ConversationScrollDown,
    NewConversation,
    EscNewConversation,
    EnterNewConversation,
    UpNewConversation,
    DownNewConversation,
}

#[derive(Debug)]
pub struct HotkeyMap {
    pub keymap: HashMap<(KeyCode, KeyModifiers), HotkeyAction>,
}

impl Default for HotkeyMap {
    fn default() -> Self {
        let mut keymap = HashMap::new();
        keymap.insert(
            (KeyCode::Char('t'), KeyModifiers::NONE),
            HotkeyAction::IncrementTempurature,
        );
        keymap.insert(
            (KeyCode::Char('T'), KeyModifiers::SHIFT),
            HotkeyAction::DecrementTempurature,
        );
        keymap.insert(
            (KeyCode::Char('p'), KeyModifiers::NONE),
            HotkeyAction::IncrementTopP,
        );
        keymap.insert(
            (KeyCode::Char('P'), KeyModifiers::SHIFT),
            HotkeyAction::DecrementTopP,
        );
        keymap.insert(
            (KeyCode::Char('f'), KeyModifiers::NONE),
            HotkeyAction::IncrementFrequencyPenalty,
        );
        keymap.insert(
            (KeyCode::Char('F'), KeyModifiers::SHIFT),
            HotkeyAction::DecrementFrequencyPenalty,
        );
        keymap.insert(
            (KeyCode::Char('r'), KeyModifiers::NONE),
            HotkeyAction::IncrementPresencePenalty,
        );
        keymap.insert(
            (KeyCode::Char('R'), KeyModifiers::SHIFT),
            HotkeyAction::DecrementPresencePenalty,
        );
        keymap.insert(
            (KeyCode::Enter, KeyModifiers::ALT),
            HotkeyAction::SendPrompt,
        );
        keymap.insert(
            (KeyCode::Char('e'), KeyModifiers::ALT),
            HotkeyAction::GetMessageFromEditor,
        );
        keymap.insert(
            (KeyCode::Char('q'), KeyModifiers::CONTROL),
            HotkeyAction::QuitProgram,
        );
        keymap.insert(
            (KeyCode::BackTab, KeyModifiers::SHIFT),
            HotkeyAction::NextTab,
        );
        for keymod in [
            KeyModifiers::NONE,
            KeyModifiers::SHIFT,
            KeyModifiers::CONTROL,
            KeyModifiers::ALT,
        ] {
            keymap.insert((KeyCode::F(1), keymod), HotkeyAction::ViewConversationTab);
            keymap.insert((KeyCode::F(2), keymod), HotkeyAction::ViewConfigTab);
        }
        keymap.insert(
            (KeyCode::PageUp, KeyModifiers::NONE),
            HotkeyAction::DebugLogsScrollUp,
        );
        keymap.insert(
            (KeyCode::PageDown, KeyModifiers::NONE),
            HotkeyAction::DebugLogsScrollDown,
        );
        keymap.insert(
            (KeyCode::PageUp, KeyModifiers::CONTROL),
            HotkeyAction::ConversationScrollUp,
        );
        keymap.insert(
            (KeyCode::PageDown, KeyModifiers::CONTROL),
            HotkeyAction::ConversationScrollDown,
        );
        keymap.insert(
            (KeyCode::Char('n'), KeyModifiers::CONTROL),
            HotkeyAction::NewConversation,
        );
        keymap.insert(
            (KeyCode::Esc, KeyModifiers::NONE),
            HotkeyAction::EscNewConversation,
        );
        keymap.insert(
            (KeyCode::Enter, KeyModifiers::NONE),
            HotkeyAction::EnterNewConversation,
        );
        keymap.insert(
            (KeyCode::Up, KeyModifiers::NONE),
            HotkeyAction::UpNewConversation,
        );
        keymap.insert(
            (KeyCode::Down, KeyModifiers::NONE),
            HotkeyAction::DownNewConversation,
        );
        HotkeyMap { keymap }
    }
}

pub enum HandleEventResult {
    None,
    Redraw,
    Quit,
}

pub async fn handle(
    timeout: u64,
    state: &mut State,
    keymap: &HotkeyMap,
) -> Result<HandleEventResult> {
    if !event::poll(std::time::Duration::from_millis(timeout)).context("poll terminal events")? {
        return Ok(HandleEventResult::None);
    };
    let terminal_event = event::read().context("read terminal event")?;
    match terminal_event {
        Event::Key(key_event) => return handle_keys(key_event, state, keymap).await,
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
    keymap: &HotkeyMap,
) -> Result<HandleEventResult> {
    if key_event.kind != KeyEventKind::Press {
        return Ok(HandleEventResult::None);
    }
    let is_hotkey = keymap.keymap.get(&(key_event.code, key_event.modifiers));
    match (is_hotkey, key_event.code, key_event.modifiers) {
        (Some(hotkey_action), _, _) => match hotkey_action {
            HotkeyAction::QuitProgram => return Ok(HandleEventResult::Quit),
            HotkeyAction::NextTab => state.tab = state.tab.next_tab(),
            HotkeyAction::ViewConversationTab => state.tab = ViewTab::Conversation,
            HotkeyAction::ViewConfigTab => state.tab = ViewTab::Config,
            _ => {
                match state.tab {
                    ViewTab::Conversation => {
                        return handle_conversation_keys(key_event, state, keymap)
                            .await
                            .context("handle conversation keys")
                    }
                    ViewTab::NewConversation => {
                        handle_new_conversation_keys(key_event, state, keymap)
                    }
                    ViewTab::Config => handle_config_keys(key_event, state, keymap),
                };
            }
        },
        _ => {
            match state.tab {
                ViewTab::Conversation => {
                    return handle_conversation_keys(key_event, state, keymap)
                        .await
                        .context("handle conversation keys")
                }
                ViewTab::NewConversation => handle_new_conversation_keys(key_event, state, keymap),
                ViewTab::Config => handle_config_keys(key_event, state, keymap),
            };
        }
    };
    Ok(HandleEventResult::None)
}

fn handle_config_keys(
    key_event: KeyEvent,
    state: &mut State,
    keymap: &HotkeyMap,
) -> HandleEventResult {
    let is_hotkey = keymap.keymap.get(&(key_event.code, key_event.modifiers));
    match (is_hotkey, key_event.code, key_event.modifiers) {
        (Some(hotkey_action), _, _) => match *hotkey_action {
            HotkeyAction::DebugLogsScrollUp => {
                state.debug_logs_scroll = state.debug_logs_scroll.saturating_sub(1);
            }
            HotkeyAction::DebugLogsScrollDown => {
                state.debug_logs_scroll = state.debug_logs_scroll.saturating_add(1);
            }
            // HotkeyAction::ToggleDebug => ui_state.debug = !ui_state.debug,
            HotkeyAction::IncrementTempurature => state.config.chat.temperature += 0.05,
            HotkeyAction::DecrementTempurature => state.config.chat.temperature -= 0.05,
            HotkeyAction::IncrementTopP => state.config.chat.top_p += 0.05,
            HotkeyAction::DecrementTopP => state.config.chat.top_p -= 0.05,
            HotkeyAction::IncrementFrequencyPenalty => state.config.chat.frequency_penalty += 0.05,
            HotkeyAction::DecrementFrequencyPenalty => state.config.chat.frequency_penalty -= 0.05,
            HotkeyAction::IncrementPresencePenalty => state.config.chat.presence_penalty += 0.05,
            HotkeyAction::DecrementPresencePenalty => state.config.chat.presence_penalty -= 0.05,
            _ => (),
        },
        _ => (),
    };
    HandleEventResult::None
}

fn handle_new_conversation_keys(
    key_event: KeyEvent,
    state: &mut State,
    keymap: &HotkeyMap,
) -> HandleEventResult {
    let is_hotkey = keymap.keymap.get(&(key_event.code, key_event.modifiers));
    match (is_hotkey, key_event.code, key_event.modifiers) {
        (Some(hotkey_action), _, _) => match *hotkey_action {
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
        },
        _ => (),
    };
    HandleEventResult::None
}

async fn handle_conversation_keys(
    key_event: KeyEvent,
    state: &mut State,
    keymap: &HotkeyMap,
) -> Result<HandleEventResult> {
    let is_hotkey = keymap.keymap.get(&(key_event.code, key_event.modifiers));
    match (is_hotkey, key_event.code, key_event.modifiers) {
        (Some(hotkey_action), _, _) => match *hotkey_action {
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
        },
        _ => {
            state.prompt_textarea.input(key_event);
        }
    };
    Ok(HandleEventResult::None)
}
