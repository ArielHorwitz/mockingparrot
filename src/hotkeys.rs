use std::collections::{hash_map, HashMap};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};

pub type HotkeyActionConfig = Option<Vec<HotkeyEvent>>;
pub type HotkeyConfig = HashMap<HotkeyAction, HotkeyActionConfig>;
pub type HotkeyMap = HashMap<KeyEvent, HotkeyAction>;

#[derive(Debug, Deserialize, Hash, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
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

const DEFAULT_HOTKEY_CONFIG: [(HotkeyAction, (KeyCode, KeyModifiers)); 29] = [
    // NextTab (default: Shift+BackTab)
    (
        HotkeyAction::NextTab,
        (KeyCode::BackTab, KeyModifiers::SHIFT),
    ),
    // IncrementTempurature (default: t)
    (
        HotkeyAction::IncrementTempurature,
        (KeyCode::Char('t'), KeyModifiers::empty()),
    ),
    // DecrementTempurature (default: Shift+T)
    (
        HotkeyAction::DecrementTempurature,
        (KeyCode::Char('T'), KeyModifiers::SHIFT),
    ),
    // IncrementTopP (default: p)
    (
        HotkeyAction::IncrementTopP,
        (KeyCode::Char('p'), KeyModifiers::empty()),
    ),
    // DecrementTopP (default: Shift+P)
    (
        HotkeyAction::DecrementTopP,
        (KeyCode::Char('P'), KeyModifiers::SHIFT),
    ),
    // IncrementFrequencyPenalty (default: f)
    (
        HotkeyAction::IncrementFrequencyPenalty,
        (KeyCode::Char('f'), KeyModifiers::empty()),
    ),
    // DecrementFrequencyPenalty (default: Shift+F)
    (
        HotkeyAction::DecrementFrequencyPenalty,
        (KeyCode::Char('F'), KeyModifiers::SHIFT),
    ),
    // IncrementPresencePenalty (default: r)
    (
        HotkeyAction::IncrementPresencePenalty,
        (KeyCode::Char('r'), KeyModifiers::empty()),
    ),
    // DecrementPresencePenalty (default: Shift+R)
    (
        HotkeyAction::DecrementPresencePenalty,
        (KeyCode::Char('R'), KeyModifiers::SHIFT),
    ),
    // SendPrompt (default: Alt+Enter)
    (
        HotkeyAction::SendPrompt,
        (KeyCode::Enter, KeyModifiers::ALT),
    ),
    // GetMessageFromEditor (default: Alt+e)
    (
        HotkeyAction::GetMessageFromEditor,
        (KeyCode::Char('e'), KeyModifiers::ALT),
    ),
    // ViewConfigTab (default: <Any>+F2)
    (
        HotkeyAction::ViewConfigTab,
        (KeyCode::F(2), KeyModifiers::CONTROL),
    ),
    (
        HotkeyAction::ViewConfigTab,
        (KeyCode::F(2), KeyModifiers::SHIFT),
    ),
    (
        HotkeyAction::ViewConfigTab,
        (KeyCode::F(2), KeyModifiers::ALT),
    ),
    (
        HotkeyAction::ViewConfigTab,
        (KeyCode::F(2), KeyModifiers::NONE),
    ),
    // ViewConversationTab (default: <Any>+F1)
    (
        HotkeyAction::ViewConversationTab,
        (KeyCode::F(1), KeyModifiers::CONTROL),
    ),
    (
        HotkeyAction::ViewConversationTab,
        (KeyCode::F(1), KeyModifiers::SHIFT),
    ),
    (
        HotkeyAction::ViewConversationTab,
        (KeyCode::F(1), KeyModifiers::ALT),
    ),
    (
        HotkeyAction::ViewConversationTab,
        (KeyCode::F(1), KeyModifiers::NONE),
    ),
    // QuitProgram (default: Ctrl+q)
    (
        HotkeyAction::QuitProgram,
        (KeyCode::Char('q'), KeyModifiers::CONTROL),
    ),
    // DebugLogsScrollUp (default: PageUp)
    (
        HotkeyAction::DebugLogsScrollUp,
        (KeyCode::PageUp, KeyModifiers::NONE),
    ),
    // DebugLogsScrollDown (default: PageDown)
    (
        HotkeyAction::DebugLogsScrollDown,
        (KeyCode::PageDown, KeyModifiers::NONE),
    ),
    // ConversationScrollUp (default: Ctrl+PageUp)
    (
        HotkeyAction::ConversationScrollUp,
        (KeyCode::PageUp, KeyModifiers::CONTROL),
    ),
    // ConversationScrollDown (default: Ctrl+PageDown)
    (
        HotkeyAction::ConversationScrollDown,
        (KeyCode::PageDown, KeyModifiers::CONTROL),
    ),
    // NewConversation (default: Ctrl+n)
    (
        HotkeyAction::NewConversation,
        (KeyCode::Char('n'), KeyModifiers::CONTROL),
    ),
    // EscNewConversation (default: Esc)
    (
        HotkeyAction::EscNewConversation,
        (KeyCode::Esc, KeyModifiers::NONE),
    ),
    // EnterNewConversation (default: Enter)
    (
        HotkeyAction::EnterNewConversation,
        (KeyCode::Enter, KeyModifiers::NONE),
    ),
    // UpNewConversation (default: Up)
    (
        HotkeyAction::UpNewConversation,
        (KeyCode::Up, KeyModifiers::NONE),
    ),
    // DownNewConversation (default: Down)
    (
        HotkeyAction::DownNewConversation,
        (KeyCode::Down, KeyModifiers::NONE),
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
