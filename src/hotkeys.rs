use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};

pub type HotkeyActionConfig = Option<Vec<HotkeyEvent>>;
pub type HotkeysConfig = HashMap<HotkeyAction, HotkeyActionConfig>;
pub type HotkeysMap = HashMap<KeyEvent, HotkeyAction>;

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
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }
    pub fn to_key_event(self) -> KeyEvent {
        KeyEvent::new(self.code, self.modifiers)
    }
}

pub fn config_to_map(config_map: HotkeysConfig) -> HotkeysMap {
    // First, merge config map with default hotkey map
    let merged_map: HotkeysConfig = get_default_config().into_iter().chain(config_map).collect();

    // Then, invert config map to create hotkey map for program
    let mut map = HashMap::new();
    merged_map
        .into_iter()
        .filter(|(_, v)| v.is_some())
        .map(|(k, v)| {
            let event_list: Vec<KeyEvent> = v
                .unwrap()
                .into_iter()
                .map(|hk_event| hk_event.to_key_event())
                .collect();
            for e in event_list {
                map.insert(e, k);
            }
        })
        .for_each(|_x| {}); // no-op to consume iterator
    map
}

pub fn get_default_config() -> HotkeysConfig {
    let mut default_config = HashMap::new();

    // NextTab (default: Shift+BackTab)
    default_config.insert(
        HotkeyAction::NextTab,
        Some(vec![HotkeyEvent::new(
            KeyCode::BackTab,
            KeyModifiers::SHIFT,
        )]),
    );
    // IncrementTempurature (default: t)
    default_config.insert(
        HotkeyAction::IncrementTempurature,
        Some(vec![HotkeyEvent::new(
            KeyCode::Char('t'),
            KeyModifiers::empty(),
        )]),
    );
    // DecrementTempurature (default: Shift+T)
    default_config.insert(
        HotkeyAction::DecrementTempurature,
        Some(vec![HotkeyEvent::new(
            KeyCode::Char('T'),
            KeyModifiers::SHIFT,
        )]),
    );
    // IncrementTopP (default: p)
    default_config.insert(
        HotkeyAction::IncrementTopP,
        Some(vec![HotkeyEvent::new(
            KeyCode::Char('p'),
            KeyModifiers::empty(),
        )]),
    );
    // DecrementTopP (default: Shift+P)
    default_config.insert(
        HotkeyAction::DecrementTopP,
        Some(vec![HotkeyEvent::new(
            KeyCode::Char('P'),
            KeyModifiers::SHIFT,
        )]),
    );
    // IncrementFrequencyPenalty (default: f)
    default_config.insert(
        HotkeyAction::IncrementFrequencyPenalty,
        Some(vec![HotkeyEvent::new(
            KeyCode::Char('f'),
            KeyModifiers::empty(),
        )]),
    );
    // DecrementFrequencyPenalty (default: Shift+F)
    default_config.insert(
        HotkeyAction::DecrementFrequencyPenalty,
        Some(vec![HotkeyEvent::new(
            KeyCode::Char('F'),
            KeyModifiers::SHIFT,
        )]),
    );
    // IncrementPresencePenalty (default: r)
    default_config.insert(
        HotkeyAction::IncrementPresencePenalty,
        Some(vec![HotkeyEvent::new(
            KeyCode::Char('r'),
            KeyModifiers::empty(),
        )]),
    );
    // DecrementPresencePenalty (default: Shift+R)
    default_config.insert(
        HotkeyAction::DecrementPresencePenalty,
        Some(vec![HotkeyEvent::new(
            KeyCode::Char('R'),
            KeyModifiers::SHIFT,
        )]),
    );
    // SendPrompt (default: Alt+Enter)
    default_config.insert(
        HotkeyAction::SendPrompt,
        Some(vec![HotkeyEvent::new(KeyCode::Enter, KeyModifiers::ALT)]),
    );
    // GetMessageFromEditor (default: Alt+e)
    default_config.insert(
        HotkeyAction::GetMessageFromEditor,
        Some(vec![HotkeyEvent::new(
            KeyCode::Char('e'),
            KeyModifiers::ALT,
        )]),
    );
    // ViewConfigTab (default: <Any>+F2)
    default_config.insert(
        HotkeyAction::ViewConfigTab,
        Some(vec![
            HotkeyEvent::new(KeyCode::F(2), KeyModifiers::CONTROL),
            HotkeyEvent::new(KeyCode::F(2), KeyModifiers::SHIFT),
            HotkeyEvent::new(KeyCode::F(2), KeyModifiers::ALT),
            HotkeyEvent::new(KeyCode::F(2), KeyModifiers::NONE),
        ]),
    );
    // ViewConversationTab (default: <Any>+F1)
    default_config.insert(
        HotkeyAction::ViewConversationTab,
        Some(vec![
            HotkeyEvent::new(KeyCode::F(1), KeyModifiers::CONTROL),
            HotkeyEvent::new(KeyCode::F(1), KeyModifiers::SHIFT),
            HotkeyEvent::new(KeyCode::F(1), KeyModifiers::ALT),
            HotkeyEvent::new(KeyCode::F(1), KeyModifiers::NONE),
        ]),
    );
    // QuitProgram (default: Ctrl+q)
    default_config.insert(
        HotkeyAction::QuitProgram,
        Some(vec![HotkeyEvent::new(
            KeyCode::Char('q'),
            KeyModifiers::CONTROL,
        )]),
    );
    // DebugLogsScrollUp (default: PageUp)
    default_config.insert(
        HotkeyAction::DebugLogsScrollUp,
        Some(vec![HotkeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE)]),
    );
    // DebugLogsScrollDown (default: PageDown)
    default_config.insert(
        HotkeyAction::DebugLogsScrollDown,
        Some(vec![HotkeyEvent::new(
            KeyCode::PageDown,
            KeyModifiers::NONE,
        )]),
    );
    // ConversationScrollUp (default: Ctrl+PageUp)
    default_config.insert(
        HotkeyAction::ConversationScrollUp,
        Some(vec![HotkeyEvent::new(
            KeyCode::PageUp,
            KeyModifiers::CONTROL,
        )]),
    );
    // ConversationScrollDown (default: Ctrl+PageDown)
    default_config.insert(
        HotkeyAction::ConversationScrollDown,
        Some(vec![HotkeyEvent::new(
            KeyCode::PageDown,
            KeyModifiers::CONTROL,
        )]),
    );
    // NewConversation (default: Ctrl+n)
    default_config.insert(
        HotkeyAction::NewConversation,
        Some(vec![HotkeyEvent::new(
            KeyCode::Char('n'),
            KeyModifiers::CONTROL,
        )]),
    );
    // EscNewConversation (default: Esc)
    default_config.insert(
        HotkeyAction::EscNewConversation,
        Some(vec![HotkeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)]),
    );
    // EnterNewConversation (default: Enter)
    default_config.insert(
        HotkeyAction::EnterNewConversation,
        Some(vec![HotkeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)]),
    );
    // UpNewConversation (default: Up)
    default_config.insert(
        HotkeyAction::UpNewConversation,
        Some(vec![HotkeyEvent::new(KeyCode::Up, KeyModifiers::NONE)]),
    );
    // DownNewConversation (default: Down)
    default_config.insert(
        HotkeyAction::DownNewConversation,
        Some(vec![HotkeyEvent::new(KeyCode::Down, KeyModifiers::NONE)]),
    );

    default_config
}
