use crate::app::actions;
use crate::app::focus::{Chat as ChatFocus, Config as ConfigFocus, Scope, Tab as TabFocus};
use crate::app::hotkeys::HotkeyAction;
use crate::app::state::State;
use crate::chat::{Conversation, Message};
use anyhow::{Context, Result};
use ratatui::crossterm::event::{self, Event, KeyEvent, KeyEventKind};

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
    let scope = state.ui.focus.get_scope();
    let hotkey_action_option = state.hotkey_map.get(&key_event).copied();
    match (scope, hotkey_action_option) {
        // Global hotkeys
        (_, Some(HotkeyAction::QuitProgram)) => return Ok(HandleEventResult::Quit),
        (_, Some(HotkeyAction::CycleTab)) => state.ui.focus.cycle_tab_next(),
        (_, Some(HotkeyAction::CycleBackTab)) => state.ui.focus.cycle_tab_prev(),
        // Scoped hotkeys
        (Scope::Chat(chat_focus), hotkey_action_option) => {
            return handle_chat(hotkey_action_option, state, chat_focus, key_event).await;
        }
        (Scope::Config(config_focus), Some(hotkey_action)) => {
            return handle_config(hotkey_action, config_focus, state);
        }
        (Scope::Debug, Some(hotkey_action)) => handle_debug(hotkey_action, state),
        _ => (),
    }
    Ok(HandleEventResult::None)
}

async fn handle_chat(
    hotkey_action_option: Option<HotkeyAction>,
    state: &mut State,
    chat_focus: ChatFocus,
    key_event: KeyEvent,
) -> Result<HandleEventResult> {
    match (chat_focus, hotkey_action_option) {
        // Focus-independent hotkeys
        (_, Some(HotkeyAction::Confirm)) => {
            let text = state.ui.prompt_textarea.lines().join("\n");
            if text.trim().is_empty() {
                state.set_status_bar_text("Cannot send empty message.");
                return Ok(HandleEventResult::None);
            }
            let message = Message::new_user_message(text);
            state.get_active_conversation_mut()?.add_message(message);
            state.ui.focus.chat = ChatFocus::Messages;
            actions::do_prompt(state).await?;
            state
                .save_conversations_to_disk()
                .context("save conversations")?;
        }
        (_, Some(HotkeyAction::Edit)) => {
            let initial_text = state.ui.prompt_textarea.lines().join("\n");
            let message_text = actions::get_message_text_from_editor(state, initial_text.as_str())
                .context("get message text from editor")?;
            state.ui.prompt_textarea.select_all();
            state.ui.prompt_textarea.cut();
            state.ui.prompt_textarea.insert_str(&message_text);
            return Ok(HandleEventResult::Redraw);
        }
        (_, Some(HotkeyAction::New)) => {
            state.ui.focus.chat = ChatFocus::New;
        }
        (_, Some(HotkeyAction::Open)) => {
            state.ui.focus.chat = ChatFocus::History;
        }
        // Scope-dependent hotkeys
        (ChatFocus::New, Some(hotkey_action)) => {
            handle_new_conversation(hotkey_action, state);
        }
        (ChatFocus::History, Some(hotkey_action)) => {
            handle_chat_history(hotkey_action, state);
        }
        (ChatFocus::Messages, Some(hotkey_action)) => {
            handle_conversation(hotkey_action, state).context("handle conversation message")?;
        }
        (ChatFocus::Prompt, _) => {
            handle_conversation_prompt(hotkey_action_option, key_event, state)
                .context("handle conversation prompt")?;
        }
        _ => (),
    }
    Ok(HandleEventResult::None)
}

fn handle_conversation(hotkey_action: HotkeyAction, state: &mut State) -> Result<()> {
    match hotkey_action {
        HotkeyAction::Select => {
            state.ui.focus.chat = ChatFocus::Prompt;
        }
        HotkeyAction::SelectionUp => {
            state.ui.conversation_scroll = state.ui.conversation_scroll.saturating_sub(1);
        }
        HotkeyAction::SelectionDown => {
            state.ui.conversation_scroll = state.ui.conversation_scroll.saturating_add(1);
        }
        HotkeyAction::ScrollUp => {
            state.ui.conversation_scroll = state.ui.conversation_scroll.saturating_sub(10);
        }
        HotkeyAction::ScrollDown => {
            state.ui.conversation_scroll = state.ui.conversation_scroll.saturating_add(10);
        }
        HotkeyAction::SelectionStart => {
            state.ui.conversation_scroll = 0;
        }
        HotkeyAction::SelectionEnd => {
            state.ui.conversation_scroll = u16::MAX;
        }
        HotkeyAction::Copy => {
            let text = state
                .get_active_conversation()
                .context("get active conversation")?
                .to_string();
            actions::export_to_clipboard(state, &text)
                .context("export conversation to clipboard")?;
            state.add_debug_log("Copied conversation to clipboard");
            state.set_status_bar_text("Copied conversation to clipboard");
        }
        _ => (),
    };
    Ok(())
}

fn handle_conversation_prompt(
    hotkey_action_option: Option<HotkeyAction>,
    key_event: KeyEvent,
    state: &mut State,
) -> Result<()> {
    match hotkey_action_option {
        Some(HotkeyAction::Cancel) => {
            state.ui.focus.chat = ChatFocus::Messages;
        }
        Some(HotkeyAction::Clear) => {
            state.ui.prompt_textarea.select_all();
            state.ui.prompt_textarea.cut();
        }
        Some(HotkeyAction::Copy) => {
            let last_message = state
                .get_active_conversation()
                .context("get active conversation")?
                .messages
                .last()
                .context("get last message")?;
            actions::export_to_clipboard(state, &last_message.content)
                .context("export last message to clipboard")?;
            state.add_debug_log("Copied last message to clipboard");
            state.set_status_bar_text("Copied last message to clipboard");
        }
        _ => {
            state.ui.prompt_textarea.input(key_event);
        }
    }
    Ok(())
}

fn handle_new_conversation(hotkey_action: HotkeyAction, state: &mut State) {
    let max_selection = state.config.system.instructions.len().saturating_sub(1);
    match hotkey_action {
        HotkeyAction::Cancel => state.ui.focus.chat = ChatFocus::Messages,
        HotkeyAction::Select => {
            if let Some(system_instructions) = state
                .config
                .system
                .instructions
                .get(state.ui.system_instruction_selection)
            {
                if state
                    .conversations
                    .first()
                    .is_some_and(Conversation::is_empty)
                {
                    state.conversations.remove(0);
                }
                let new_conversation = Conversation::new(system_instructions.message.clone());
                state.conversations.insert(0, new_conversation);
                state.ui.active_conversation_index = 0;
                state.ui.focus.chat = ChatFocus::Prompt;
            };
        }
        HotkeyAction::SelectionDown => {
            state.ui.system_instruction_selection = state
                .ui
                .system_instruction_selection
                .saturating_add(1)
                .min(max_selection);
        }
        HotkeyAction::SelectionUp => {
            state.ui.system_instruction_selection =
                state.ui.system_instruction_selection.saturating_sub(1);
        }
        HotkeyAction::ScrollDown => {
            state.ui.system_instruction_selection = state
                .ui
                .system_instruction_selection
                .saturating_add(10)
                .min(max_selection);
        }
        HotkeyAction::ScrollUp => {
            state.ui.system_instruction_selection =
                state.ui.system_instruction_selection.saturating_sub(10);
        }
        HotkeyAction::SelectionStart => {
            state.ui.system_instruction_selection = 0;
        }
        HotkeyAction::SelectionEnd => {
            state.ui.system_instruction_selection = max_selection;
        }
        _ => (),
    }
}

fn handle_chat_history(hotkey_action: HotkeyAction, state: &mut State) {
    let max_selection = state.conversations.len().saturating_sub(1);
    match hotkey_action {
        HotkeyAction::Cancel | HotkeyAction::Select => {
            state.ui.focus.chat = ChatFocus::Messages;
        }
        HotkeyAction::SelectionUp => {
            state.ui.active_conversation_index =
                state.ui.active_conversation_index.saturating_sub(1);
        }
        HotkeyAction::SelectionDown => {
            state.ui.active_conversation_index = state
                .ui
                .active_conversation_index
                .saturating_add(1)
                .min(max_selection);
        }
        HotkeyAction::SelectionStart => {
            state.ui.active_conversation_index = 0;
        }
        HotkeyAction::SelectionEnd => {
            state.ui.active_conversation_index = max_selection;
        }
        HotkeyAction::ScrollUp => {
            state.ui.active_conversation_index =
                state.ui.active_conversation_index.saturating_sub(10);
        }
        HotkeyAction::ScrollDown => {
            state.ui.active_conversation_index = state
                .ui
                .active_conversation_index
                .saturating_add(10)
                .min(max_selection);
        }
        _ => (),
    };
}

fn handle_config(
    hotkey_action: HotkeyAction,
    config_focus: ConfigFocus,
    state: &mut State,
) -> Result<HandleEventResult> {
    match (hotkey_action, config_focus) {
        (HotkeyAction::Cancel, _) => state.ui.focus.set_tab(TabFocus::Chat),
        (HotkeyAction::Edit, _) => {
            actions::edit_config_file_in_editor(state)?;
            state.reload_config()?;
            return Ok(HandleEventResult::Redraw);
        }
        (HotkeyAction::Refresh, _) => {
            state.reload_models()?;
            state.reload_config()?;
        }
        _ => (),
    };
    Ok(HandleEventResult::None)
}

fn handle_debug(hotkey_action: HotkeyAction, state: &mut State) {
    match hotkey_action {
        HotkeyAction::Cancel => state.ui.focus.set_tab(TabFocus::Chat),
        HotkeyAction::ScrollUp => {
            state.ui.debug_logs_scroll = state.ui.debug_logs_scroll.saturating_sub(1);
        }
        HotkeyAction::ScrollDown => {
            state.ui.debug_logs_scroll = state.ui.debug_logs_scroll.saturating_add(1);
        }
        _ => (),
    }
}
