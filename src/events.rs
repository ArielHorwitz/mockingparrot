use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use tui_textarea::TextArea;

#[derive(Debug)]
pub enum EventResult {
    None,
    Prompt,
    Feedback(String),
    Quit,
}

pub fn handle_events(timeout: u64, textarea: &mut TextArea) -> Result<EventResult> {
    if !event::poll(std::time::Duration::from_millis(timeout)).context("poll terminal events")? {
        return Ok(EventResult::None);
    };
    let terminal_event = event::read().context("read terminal event")?;
    match terminal_event {
        Event::Key(key_event) => handle_keys(key_event, textarea),
        Event::FocusGained => Ok(EventResult::Feedback(String::from("focus gained"))),
        Event::FocusLost => Ok(EventResult::Feedback(String::from("focus lost"))),
        Event::Mouse(ev) => Ok(EventResult::Feedback(format!("mouse {ev:#?}"))),
        Event::Paste(p) => Ok(EventResult::Feedback(format!("paste {p:#?}"))),
        Event::Resize(x, y) => Ok(EventResult::Feedback(format!("resize {x}x{y}"))),
    }
}

fn handle_keys(key_event: KeyEvent, textarea: &mut TextArea) -> Result<EventResult> {
    if key_event.kind != KeyEventKind::Press {
        return Ok(EventResult::None);
    }
    if key_event.code == KeyCode::Char('q') && key_event.modifiers.contains(KeyModifiers::CONTROL) {
        return Ok(EventResult::Quit);
    }
    if key_event.code == KeyCode::Enter && key_event.modifiers.contains(KeyModifiers::ALT) {
        return Ok(EventResult::Prompt);
    }
    textarea.input(key_event);
    Ok(EventResult::None)
}
