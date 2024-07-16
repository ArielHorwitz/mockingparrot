use std::collections::HashMap;

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
    New,
    Clear,
    SelectionUp,
    SelectionDown,
    ScrollUp,
    ScrollDown,
    CycleTab,
    ViewConversationTab,
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
pub fn get_hotkey_config(config_map: HotkeyConfig) -> HotkeyMap {
    let mut map = HashMap::new();
    for (k, v) in config_map {
        let event_list: Vec<KeyEvent> = v
            .unwrap_or(Vec::new())
            .into_iter()
            .map(HotkeyEvent::to_key_event)
            .collect();
        for e in event_list {
            map.insert(e, k);
        }
    }
    map
}
