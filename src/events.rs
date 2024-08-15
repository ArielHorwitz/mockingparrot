use crate::api::GptMessage;
use crate::hotkeys::HotkeyAction;
use crate::state::focus::{
    Config as ConfigFocus, Conversation as ConversationFocus, Scope, Tab as TabFocus,
};
use crate::state::{Conversation, State};
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
    match (
        state.ui.focus.get_scope(),
        state.hotkey_map.get(&key_event).copied(),
    ) {
        // Global hotkeys
        (_, Some(HotkeyAction::QuitProgram)) => return Ok(HandleEventResult::Quit),
        (_, Some(HotkeyAction::ViewConversationTab)) => {
            state.ui.focus.set_tab(TabFocus::Conversation);
        }
        (_, Some(HotkeyAction::ViewConversationHistoryTab)) => {
            state.ui.focus.set_tab(TabFocus::ConversationHistory);
        }
        (_, Some(HotkeyAction::ViewConfigTab)) => state.ui.focus.set_tab(TabFocus::Config),
        (_, Some(HotkeyAction::ViewDebugTab)) => state.ui.focus.set_tab(TabFocus::Debug),
        (_, Some(HotkeyAction::CycleTab)) => state.ui.focus.cycle_tab(),
        // Scoped hotkeys
        (Scope::Conversation(conversation_focus), hotkey_action_option) => {
            return handle_conversation(hotkey_action_option, state, conversation_focus, key_event)
                .await;
        }
        (Scope::Debug, Some(hotkey_action)) => handle_debug(hotkey_action, state),
        (Scope::ConversationHistory, Some(hotkey_action)) => {
            handle_conversation_history(hotkey_action, state);
        }
        (Scope::NewConversation, Some(hotkey_action)) => {
            handle_new_conversation(hotkey_action, state);
        }
        (Scope::Config(config_focus), Some(hotkey_action)) => {
            handle_config(hotkey_action, config_focus, state);
        }
        _ => (),
    }
    Ok(HandleEventResult::None)
}

async fn handle_conversation(
    hotkey_action_option: Option<HotkeyAction>,
    state: &mut State,
    conversation_focus: ConversationFocus,
    key_event: KeyEvent,
) -> Result<HandleEventResult> {
    match (conversation_focus, hotkey_action_option) {
        // Focus-independent hotkeys
        (_, Some(HotkeyAction::SendPrompt)) => {
            let text = state.ui.prompt_textarea.lines().join("\n");
            if text.trim().is_empty() {
                state.set_status_bar_text("Cannot send empty message.");
                return Ok(HandleEventResult::None);
            }
            let message = GptMessage::new_user_message(text);
            state.get_active_conversation_mut()?.add_message(message);
            state.ui.focus.conversation = ConversationFocus::Messages;
            do_prompt(state).await?;
        }
        (_, Some(HotkeyAction::GetMessageFromEditor)) => {
            let initial_text = state.ui.prompt_textarea.lines().join("\n");
            let message_text = get_message_text_from_editor(state, initial_text.as_str())
                .context("get message text from editor")?;
            state.ui.prompt_textarea.select_all();
            state.ui.prompt_textarea.cut();
            state.ui.prompt_textarea.insert_str(&message_text);
            return Ok(HandleEventResult::Redraw);
        }
        // Prompt focus
        (ConversationFocus::Prompt, Some(HotkeyAction::Cancel)) => {
            state.ui.focus.conversation = ConversationFocus::Messages;
        }
        (ConversationFocus::Prompt, Some(HotkeyAction::Clear)) => {
            state.ui.prompt_textarea.select_all();
            state.ui.prompt_textarea.cut();
        }
        (ConversationFocus::Prompt, _) => {
            state.ui.prompt_textarea.input(key_event);
        }
        // Conversation messages focus
        (ConversationFocus::Messages, Some(hotkey_action)) => {
            handle_conversation_messages(hotkey_action, state);
        }
        _ => (),
    }
    Ok(HandleEventResult::None)
}

fn handle_conversation_history(hotkey_action: HotkeyAction, state: &mut State) {
    match hotkey_action {
        HotkeyAction::Select => {
            state.ui.focus.conversation = ConversationFocus::Prompt;
            state.ui.focus.set_tab(TabFocus::Conversation);
        }
        HotkeyAction::SelectionUp => {
            state.ui.active_conversation_index =
                state.ui.active_conversation_index.saturating_sub(1);
        }
        HotkeyAction::SelectionDown => {
            state.ui.active_conversation_index =
                state.ui.active_conversation_index.saturating_add(1);
        }
        HotkeyAction::ScrollUp => {
            state.ui.active_conversation_index =
                state.ui.active_conversation_index.saturating_sub(10);
        }
        HotkeyAction::ScrollDown => {
            state.ui.active_conversation_index =
                state.ui.active_conversation_index.saturating_add(10);
        }
        HotkeyAction::New => {
            state.ui.focus.set_tab(TabFocus::NewConversation);
        }
        _ => (),
    };
}

fn handle_conversation_messages(hotkey_action: HotkeyAction, state: &mut State) {
    match hotkey_action {
        HotkeyAction::Select => {
            state.ui.focus.conversation = ConversationFocus::Prompt;
        }
        HotkeyAction::ScrollUp => {
            state.ui.conversation_scroll = state.ui.conversation_scroll.saturating_sub(1);
        }
        HotkeyAction::ScrollDown => {
            state.ui.conversation_scroll = state.ui.conversation_scroll.saturating_add(1);
        }
        HotkeyAction::New => {
            state.ui.focus.set_tab(TabFocus::NewConversation);
        }
        _ => (),
    };
}

fn handle_new_conversation(hotkey_action: HotkeyAction, state: &mut State) {
    match hotkey_action {
        HotkeyAction::Cancel => state.ui.focus.set_tab(TabFocus::Conversation),
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
                state.ui.focus.set_tab(TabFocus::Conversation);
            };
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
}

fn handle_config(hotkey_action: HotkeyAction, config_focus: ConfigFocus, state: &mut State) {
    match (hotkey_action, config_focus) {
        (HotkeyAction::SelectionUp, _) => {
            state.ui.focus.config = state.ui.focus.config.prev_cycle();
        }
        (HotkeyAction::SelectionDown, _) => {
            state.ui.focus.config = state.ui.focus.config.next_cycle();
        }
        (HotkeyAction::Increment, ConfigFocus::MaxTokens) => {
            state.config.chat.max_tokens.increment();
        }
        (HotkeyAction::Increment, ConfigFocus::Temperature) => {
            state.config.chat.temperature.increment();
        }
        (HotkeyAction::Increment, ConfigFocus::TopP) => state.config.chat.top_p.increment(),
        (HotkeyAction::Increment, ConfigFocus::FrequencyPenalty) => {
            state.config.chat.frequency_penalty.increment();
        }
        (HotkeyAction::Increment, ConfigFocus::PresencePenalty) => {
            state.config.chat.presence_penalty.increment();
        }
        (HotkeyAction::Decrement, ConfigFocus::MaxTokens) => {
            state.config.chat.max_tokens.decrement();
        }
        (HotkeyAction::Decrement, ConfigFocus::Temperature) => {
            state.config.chat.temperature.decrement();
        }
        (HotkeyAction::Decrement, ConfigFocus::TopP) => state.config.chat.top_p.decrement(),
        (HotkeyAction::Decrement, ConfigFocus::FrequencyPenalty) => {
            state.config.chat.frequency_penalty.decrement();
        }
        (HotkeyAction::Decrement, ConfigFocus::PresencePenalty) => {
            state.config.chat.presence_penalty.decrement();
        }
        _ => (),
    };
}

fn handle_debug(hotkey_action: HotkeyAction, state: &mut State) {
    match hotkey_action {
        HotkeyAction::ScrollUp => {
            state.ui.debug_logs_scroll = state.ui.debug_logs_scroll.saturating_sub(1);
        }
        HotkeyAction::ScrollDown => {
            state.ui.debug_logs_scroll = state.ui.debug_logs_scroll.saturating_add(1);
        }
        _ => (),
    }
}
