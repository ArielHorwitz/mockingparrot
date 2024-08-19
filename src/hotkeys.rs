use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, ModifierKeyCode};
use serde::de::Visitor;
use serde::{Deserialize, Serialize};

pub type HotkeyActionConfig = Option<Vec<HotkeyEvent>>;
pub type HotkeyConfig = HashMap<HotkeyAction, HotkeyActionConfig>;
pub type HotkeyMap = HashMap<KeyEvent, HotkeyAction>;

#[derive(Debug, Deserialize, Hash, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HotkeyAction {
    QuitProgram,
    New,
    Open,
    Edit,
    Copy,
    Clear,
    Refresh,
    Confirm,
    Select,
    Cancel,
    SelectionUp,
    SelectionDown,
    SelectionStart,
    SelectionEnd,
    ScrollUp,
    ScrollDown,
    Increment,
    Decrement,
    CycleTab,
    CycleBackTab,
    ViewConversationTab,
    ViewConfigTab,
    ViewDebugTab,
}

#[derive(Debug, Serialize, Hash, Clone, Copy, PartialEq, Eq)]
pub struct HotkeyEvent {
    code: KeyCode,
    modifiers: KeyModifiers,
}

impl<'de> Deserialize<'de> for HotkeyEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(HotkeyEventVisitor)
    }
}

struct HotkeyEventVisitor;

impl<'de> Visitor<'de> for HotkeyEventVisitor {
    type Value = HotkeyEvent;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string representing a hotkey event")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let mut parts: Vec<&str> = value.split_whitespace().collect();

        let code_name = parts
            .pop()
            .ok_or(serde::de::Error::custom("missing key code"))?
            .to_lowercase();
        let mut code_chars = code_name.chars();
        let first_char = code_chars
            .next()
            .ok_or(serde::de::Error::custom("empty key code"))?;
        let next_chars: String = code_chars.collect();
        let single_char = next_chars.is_empty();

        let code = if single_char {
            KeyCode::Char(first_char)
        } else if !single_char && first_char == 'f' {
            let number = str::parse::<u8>(&next_chars)
                .map_err(|_| serde::de::Error::custom("parse number after f"))?;
            KeyCode::F(number)
        } else {
            match code_name.as_str() {
                "backspace" => KeyCode::Backspace,
                "enter" => KeyCode::Enter,
                "left" => KeyCode::Left,
                "right" => KeyCode::Right,
                "up" => KeyCode::Up,
                "down" => KeyCode::Down,
                "home" => KeyCode::Home,
                "end" => KeyCode::End,
                "pageup" | "pgup" => KeyCode::PageUp,
                "pagedown" | "pgdn" => KeyCode::PageDown,
                "tab" => KeyCode::Tab,
                "backtab" => KeyCode::BackTab,
                "delete" | "del" => KeyCode::Delete,
                "insert" | "ins" => KeyCode::Insert,
                "escape" | "esc" => KeyCode::Esc,
                "capslock" => KeyCode::CapsLock,
                "scrolllock" => KeyCode::ScrollLock,
                "numlock" => KeyCode::NumLock,
                "printscreen" => KeyCode::PrintScreen,
                "pause" => KeyCode::Pause,
                "menu" => KeyCode::Menu,
                "keypadbegin" => KeyCode::KeypadBegin,
                "control" | "ctrl" | "lcontrol" | "lctrl" => {
                    KeyCode::Modifier(ModifierKeyCode::LeftControl)
                }
                "rcontrol" | "rctrl" => KeyCode::Modifier(ModifierKeyCode::RightControl),
                "alt" | "lalt" => KeyCode::Modifier(ModifierKeyCode::LeftAlt),
                "ralt" => KeyCode::Modifier(ModifierKeyCode::RightAlt),
                "shift" | "lshift" => KeyCode::Modifier(ModifierKeyCode::LeftShift),
                "rshift" => KeyCode::Modifier(ModifierKeyCode::RightShift),
                "super" | "lsuper" => KeyCode::Modifier(ModifierKeyCode::LeftSuper),
                "rsuper" => KeyCode::Modifier(ModifierKeyCode::RightSuper),
                "meta" => KeyCode::Modifier(ModifierKeyCode::LeftMeta),
                "rmeta" => KeyCode::Modifier(ModifierKeyCode::RightMeta),
                _ => {
                    return Err(serde::de::Error::custom(format!(
                        "unrecognized key: '{code_name}'"
                    )))
                }
            }
        };

        let mut modifiers = KeyModifiers::empty();
        for modifier in parts {
            modifiers |= match modifier {
                "control" | "ctrl" => KeyModifiers::CONTROL,
                "alt" => KeyModifiers::ALT,
                "shift" => KeyModifiers::SHIFT,
                "super" | "win" => KeyModifiers::SUPER,
                "meta" => KeyModifiers::META,
                _ => {
                    return Err(serde::de::Error::custom(format!(
                        "unrecognized modifier key: '{modifier}'"
                    )))
                }
            }
        }

        Ok(HotkeyEvent { code, modifiers })
    }
}

#[must_use]
pub fn get_hotkey_config(config_map: HotkeyConfig) -> HotkeyMap {
    let mut map = HashMap::new();
    for (k, v) in config_map {
        let event_list: Vec<KeyEvent> = v
            .unwrap_or(Vec::new())
            .into_iter()
            .map(|ev| KeyEvent::new(ev.code, ev.modifiers))
            .collect();
        for e in event_list {
            map.insert(e, k);
        }
    }
    map
}
