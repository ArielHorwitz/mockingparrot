use std::collections::{hash_map, HashMap};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};

pub type HotkeyActionConfig = Option<Vec<HotkeyEvent>>;
pub type HotkeyConfig = HashMap<HotkeyAction, HotkeyActionConfig>;
pub type HotkeyMap = HashMap<KeyEvent, HotkeyAction>;

#[derive(Debug, Deserialize, Hash, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HotkeyAction {
    QuitProgram,
    Select,
    Cancel,
    SelectionUp,
    SelectionDown,
    ScrollUp,
    ScrollDown,
    NextTab,
    ViewConversationTab,
    NewConversation,
    ViewConfigTab,
    ViewDebugTab,
    SendPrompt,
    GetMessageFromEditor,
    IncrementTempurature,
    DecrementTempurature,
    IncrementTopP,
    DecrementTopP,
    IncrementFrequencyPenalty,
    DecrementFrequencyPenalty,
    IncrementPresencePenalty,
    DecrementPresencePenalty,
}

#[derive(Debug, Serialize, Deserialize, Hash, Clone, Copy, PartialEq, Eq)]
pub struct HotkeyEvent {
    code: KeyCode,
    modifiers: KeyModifiers,
}

impl HotkeyEvent {
    #[must_use]
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }
    #[must_use]
    pub fn to_key_event(self) -> KeyEvent {
        KeyEvent::new(self.code, self.modifiers)
    }
}

#[must_use]
pub fn config_to_map(config_map: HotkeyConfig) -> HotkeyMap {
    // First, merge config map with default hotkey map
    let merged_map: HotkeyConfig = get_default_config().into_iter().chain(config_map).collect();

    // Then, invert config map to create hotkey map for program
    let mut map = HashMap::new();
    merged_map
        .into_iter()
        .filter(|(_, v)| v.is_some())
        .map(|(k, v)| {
            let event_list: Vec<KeyEvent> = v
                .unwrap_or(Vec::new())
                .into_iter()
                .map(HotkeyEvent::to_key_event)
                .collect();
            for e in event_list {
                map.insert(e, k);
            }
        })
        .for_each(|_x| {}); // no-op to consume iterator
    map
}

const DEFAULT_HOTKEY_CONFIG: [(HotkeyAction, (KeyCode, KeyModifiers)); 22] = [
    (
        HotkeyAction::QuitProgram,
        (KeyCode::Char('q'), KeyModifiers::CONTROL),
    ),
    (HotkeyAction::Select, (KeyCode::Enter, KeyModifiers::NONE)),
    (HotkeyAction::Cancel, (KeyCode::Esc, KeyModifiers::NONE)),
    (HotkeyAction::SelectionUp, (KeyCode::Up, KeyModifiers::NONE)),
    (
        HotkeyAction::SelectionDown,
        (KeyCode::Down, KeyModifiers::NONE),
    ),
    (
        HotkeyAction::ScrollUp,
        (KeyCode::PageUp, KeyModifiers::NONE),
    ),
    (
        HotkeyAction::ScrollDown,
        (KeyCode::PageDown, KeyModifiers::NONE),
    ),
    (
        HotkeyAction::NextTab,
        (KeyCode::BackTab, KeyModifiers::SHIFT),
    ),
    (
        HotkeyAction::ViewConversationTab,
        (KeyCode::F(1), KeyModifiers::NONE),
    ),
    (
        HotkeyAction::NewConversation,
        (KeyCode::Char('n'), KeyModifiers::CONTROL),
    ),
    (
        HotkeyAction::ViewConfigTab,
        (KeyCode::F(2), KeyModifiers::NONE),
    ),
    (
        HotkeyAction::ViewDebugTab,
        (KeyCode::F(3), KeyModifiers::NONE),
    ),
    (
        HotkeyAction::SendPrompt,
        (KeyCode::Enter, KeyModifiers::ALT),
    ),
    (
        HotkeyAction::GetMessageFromEditor,
        (KeyCode::Char('e'), KeyModifiers::ALT),
    ),
    (
        HotkeyAction::IncrementTempurature,
        (KeyCode::Char('t'), KeyModifiers::empty()),
    ),
    (
        HotkeyAction::DecrementTempurature,
        (KeyCode::Char('T'), KeyModifiers::SHIFT),
    ),
    (
        HotkeyAction::IncrementTopP,
        (KeyCode::Char('p'), KeyModifiers::empty()),
    ),
    (
        HotkeyAction::DecrementTopP,
        (KeyCode::Char('P'), KeyModifiers::SHIFT),
    ),
    (
        HotkeyAction::IncrementFrequencyPenalty,
        (KeyCode::Char('f'), KeyModifiers::empty()),
    ),
    (
        HotkeyAction::DecrementFrequencyPenalty,
        (KeyCode::Char('F'), KeyModifiers::SHIFT),
    ),
    (
        HotkeyAction::IncrementPresencePenalty,
        (KeyCode::Char('r'), KeyModifiers::empty()),
    ),
    (
        HotkeyAction::DecrementPresencePenalty,
        (KeyCode::Char('R'), KeyModifiers::SHIFT),
    ),
];

#[must_use]
pub fn get_default_config() -> HotkeyConfig {
    let mut default_config: HotkeyConfig = HashMap::new();

    for (k, v) in DEFAULT_HOTKEY_CONFIG {
        if let hash_map::Entry::Vacant(e) = default_config.entry(k) {
            e.insert(Some(vec![HotkeyEvent::new(v.0, v.1)]));
        } else {
            default_config
                .get_mut(&k)
                .unwrap_or(&mut Some(Vec::new()))
                .as_mut()
                .unwrap_or(&mut Vec::new())
                .push(HotkeyEvent::new(v.0, v.1));
        }
    }

    default_config
}
