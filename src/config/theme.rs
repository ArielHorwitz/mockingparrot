use anyhow::{Context, Result};
use ratatui::prelude::{Color, Modifier, Style};
use serde::Deserialize;
use std::{path::Path, str::FromStr};

const DEFAULT_THEME: &str = include_str!("../../templates/theme.toml");

#[derive(Debug, Clone, Copy)]
pub struct ColorMod {
    color: Color,
    modifiers: Modifier,
}

impl FromStr for ColorMod {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut parts = s.split(';');
        let color =
            Color::from_str(parts.next().context("missing color")?).context("parse color")?;
        let mut modifiers = Modifier::default();
        for modifier in parts {
            modifiers |= match modifier.to_lowercase().as_str() {
                "d" | "dim" => Modifier::DIM,
                "b" | "bold" => Modifier::BOLD,
                "i" | "italic" => Modifier::ITALIC,
                "u" | "underline" => Modifier::UNDERLINED,
                "slow" | "slow blink" | "slow-blink" => Modifier::SLOW_BLINK,
                "rapid" | "rapid blink" | "rapid-blink" => Modifier::RAPID_BLINK,
                "reversed" => Modifier::REVERSED,
                "hidden" => Modifier::HIDDEN,
                "crossed" | "crossed out" | "crossed-out" => Modifier::CROSSED_OUT,
                _ => anyhow::bail!("invalid modifier: {modifier}"),
            };
        }
        let new = Self { color, modifiers };
        Ok(new)
    }
}

impl<'de> Deserialize<'de> for ColorMod {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl ColorMod {
    fn as_fg(self) -> Style {
        Style::default().add_modifier(self.modifiers).fg(self.color)
    }

    fn as_bg(self) -> Style {
        Style::default().add_modifier(self.modifiers).bg(self.color)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Theme {
    background: ColorMod,
    background_highlighted: ColorMod,
    text: ColorMod,
    text_inactive: ColorMod,
    text_warn: ColorMod,
    name: ColorMod,
    name_inactive: ColorMod,
    title: ColorMod,
    frame: ColorMod,
    frame_inactive: ColorMod,
    tab_selected: ColorMod,
    tab_unselected: ColorMod,
    input_text: ColorMod,
    input_text_inactive: ColorMod,
    cursor: ColorMod,
    cursor_inactive: ColorMod,
}

#[allow(clippy::match_bool)]
impl Theme {
    #[must_use]
    pub fn background(&self, highlight: bool) -> Style {
        match highlight {
            true => self.background_highlighted.as_bg(),
            false => self.background.as_bg(),
        }
    }

    #[must_use]
    pub fn text(&self, active: bool) -> Style {
        match active {
            true => self.text.as_fg(),
            false => self.text_inactive.as_fg(),
        }
    }

    #[must_use]
    pub fn text_warn(&self) -> Style {
        self.text_warn.as_fg()
    }

    #[must_use]
    pub fn name(&self, active: bool) -> Style {
        match active {
            true => self.name.as_fg(),
            false => self.name_inactive.as_fg(),
        }
    }

    #[must_use]
    pub fn title(&self) -> Style {
        self.title.as_fg()
    }

    #[must_use]
    pub fn frame(&self, active: bool) -> Style {
        match active {
            true => self.frame.as_fg(),
            false => self.frame_inactive.as_fg(),
        }
    }

    #[must_use]
    pub fn tab(&self, selected: bool) -> Style {
        match selected {
            true => self.tab_selected.as_fg(),
            false => self.tab_unselected.as_fg(),
        }
    }

    #[must_use]
    pub fn input_text(&self, active: bool) -> Style {
        match active {
            true => self.input_text.as_fg(),
            false => self.input_text_inactive.as_fg(),
        }
    }

    #[must_use]
    pub fn cursor(&self, active: bool) -> Style {
        match active {
            true => self.cursor.as_bg(),
            false => self.cursor_inactive.as_bg(),
        }
    }

    pub fn generate_default(theme_file: &Path) -> Result<()> {
        if !theme_file.exists() {
            std::fs::write(theme_file, DEFAULT_THEME)
                .context("generate theme file from default template")?;
        }
        Ok(())
    }

    pub fn from_disk(theme_file: &Path) -> Result<Self> {
        let theme_file_contents = std::fs::read_to_string(theme_file).context("read theme file")?;
        toml::from_str(&theme_file_contents).context("parse theme toml")
    }
}
