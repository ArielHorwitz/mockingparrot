use crate::ui::{UiState, ViewTab};
use crate::state::State;
use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

#[derive(Debug)]
pub enum EventResult {
    None,
    Prompt,
    Feedback(String),
    Quit,
}

pub fn handle_events(timeout: u64, state: &mut State, ui_state: &mut UiState) -> Result<EventResult> {
    if !event::poll(std::time::Duration::from_millis(timeout)).context("poll terminal events")? {
        return Ok(EventResult::None);
    };
    let terminal_event = event::read().context("read terminal event")?;
    match terminal_event {
        Event::Key(key_event) => handle_keys(key_event, state, ui_state),
        Event::FocusGained => Ok(EventResult::Feedback(String::from("focus gained"))),
        Event::FocusLost => Ok(EventResult::Feedback(String::from("focus lost"))),
        Event::Mouse(ev) => Ok(EventResult::Feedback(format!("mouse {ev:#?}"))),
        Event::Paste(p) => Ok(EventResult::Feedback(format!("paste {p:#?}"))),
        Event::Resize(x, y) => Ok(EventResult::Feedback(format!("resize {x}x{y}"))),
    }
}

fn handle_keys(key_event: KeyEvent, state: &mut State, ui_state: &mut UiState) -> Result<EventResult> {
    if key_event.kind != KeyEventKind::Press {
        return Ok(EventResult::None);
    }
    ui_state.key_event_debug = format!("{:?} {:?}", key_event.modifiers, key_event.code);
    match (key_event.code, key_event.modifiers) {
        (KeyCode::Char('q'), KeyModifiers::CONTROL) => return Ok(EventResult::Quit),
        (KeyCode::BackTab, KeyModifiers::SHIFT) => ui_state.tab = ui_state.tab.next_tab(),
        (KeyCode::F(1), _) => ui_state.tab = ViewTab::Conversation,
        (KeyCode::F(2), _) => ui_state.tab = ViewTab::Config,
        _ => {
            if ui_state.tab == ViewTab::Conversation {
                match (key_event.code, key_event.modifiers) {
                    (KeyCode::Enter, KeyModifiers::ALT) => return Ok(EventResult::Prompt),
                    _ => ui_state.textarea.input(key_event),
                };
            }
            if ui_state.tab == ViewTab::Config {
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
            }
        }
    };
    Ok(EventResult::None)
}
