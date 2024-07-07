use crate::api::GptMessage;
use crate::hotkeys::HotkeyAction;
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
    if let Some(hotkey_action) = state.hotkey_map.get(&(key_event)) {
        match hotkey_action {
            HotkeyAction::QuitProgram => return Ok(HandleEventResult::Quit),
            HotkeyAction::NextTab => state.ui.tab = state.ui.tab.next_tab(),
            HotkeyAction::ViewConversationTab => state.ui.tab = ViewTab::Conversation,
            HotkeyAction::ViewConfigTab => state.ui.tab = ViewTab::Config,
            _ => {
                match state.ui.tab {
                    ViewTab::Conversation => {
                        return handle_conversation_keys(key_event, state)
                            .await
                            .context("handle conversation keys")
                    }
                    ViewTab::NewConversation => handle_new_conversation_keys(key_event, state),
                    ViewTab::Config => handle_config_keys(key_event, state),
                };
            }
        }
    } else {
        match state.ui.tab {
            ViewTab::Conversation => {
                return handle_conversation_keys(key_event, state)
                    .await
                    .context("handle conversation keys")
            }
            ViewTab::NewConversation => handle_new_conversation_keys(key_event, state),
            ViewTab::Config => handle_config_keys(key_event, state),
        };
    }
    Ok(HandleEventResult::None)
}

fn handle_config_keys(key_event: KeyEvent, state: &mut State) -> HandleEventResult {
    if let Some(hotkey_action) = state.hotkey_map.get(&(key_event)) {
        match hotkey_action {
            HotkeyAction::ScrollUp => {
                state.ui.debug_logs_scroll = state.ui.debug_logs_scroll.saturating_sub(1);
            }
            HotkeyAction::ScrollDown => {
                state.ui.debug_logs_scroll = state.ui.debug_logs_scroll.saturating_add(1);
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

fn handle_new_conversation_keys(key_event: KeyEvent, state: &mut State) -> HandleEventResult {
    if let Some(hotkey_action) = state.hotkey_map.get(&(key_event)) {
        match hotkey_action {
            HotkeyAction::Cancel => state.ui.tab = ViewTab::Conversation,
            HotkeyAction::Select => {
                if let Some(system_instructions) = state
                    .config
                    .system
                    .instructions
                    .get(state.ui.system_instruction_selection)
                {
                    state.conversation = Conversation::new(system_instructions.message.clone());
                };
                state.ui.tab = ViewTab::Conversation;
            }
            HotkeyAction::SelectionDown => {
                let new_selection = state.ui.system_instruction_selection.saturating_add(1);
                if new_selection >= state.config.system.instructions.len() {
                    state.ui.system_instruction_selection = 0;
                } else {
                    state.ui.system_instruction_selection = new_selection;
                }
            }
            HotkeyAction::SelectionUp => {
                let new_selection = state
                    .ui
                    .system_instruction_selection
                    .checked_sub(1)
                    .unwrap_or(state.config.system.instructions.len() - 1);
                state.ui.system_instruction_selection = new_selection;
            }
            _ => (),
        }
    };
    HandleEventResult::None
}

async fn handle_conversation_keys(
    key_event: KeyEvent,
    state: &mut State,
) -> Result<HandleEventResult> {
    if let Some(hotkey_action) = state.hotkey_map.get(&(key_event)) {
        match hotkey_action {
            HotkeyAction::ScrollUp => {
                state.ui.conversation_scroll = state.ui.conversation_scroll.saturating_sub(1);
            }
            HotkeyAction::ScrollDown => {
                state.ui.conversation_scroll = state.ui.conversation_scroll.saturating_add(1);
            }
            HotkeyAction::NewConversation => state.ui.tab = ViewTab::NewConversation,
            HotkeyAction::SendPrompt => {
                let text = state.ui.prompt_textarea.lines().join("\n");
                if text.trim().is_empty() {
                    state.set_status_bar_text("Cannot send empty message.");
                    return Ok(HandleEventResult::None);
                }
                let message = GptMessage::new_user_message(text);
                state.conversation.add_message(message);
                do_prompt(state).await?;
            }
            HotkeyAction::GetMessageFromEditor => {
                let initial_text = state.ui.prompt_textarea.lines().join("\n");
                let message_text =
                    get_message_text_from_editor(&state.config, initial_text.as_str())
                        .context("get message text from editor")?;
                state.ui.prompt_textarea.select_all();
                state.ui.prompt_textarea.cut();
                state.ui.prompt_textarea.insert_str(&message_text);
                return Ok(HandleEventResult::Redraw);
            }
            _ => {
                state.ui.prompt_textarea.input(key_event);
            }
        }
    } else {
        state.ui.prompt_textarea.input(key_event);
    }
    Ok(HandleEventResult::None)
}
