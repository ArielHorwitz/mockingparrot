use ratatui::prelude::Color;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Ui {
    pub layout: Layout,
    pub colors: Colors,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Layout {
    pub prompt_size: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Colors {
    pub text: ColorVariants,
    pub background: ColorVariants,
    pub frame: ColorVariants,
    pub widget: ColorVariants,
    pub cursor: ColorVariants,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ColorVariants {
    pub normal: Color,
    pub inactive: Color,
    pub highlight: Color,
    pub title: Color,
    pub warn: Color,
}

impl ColorVariants {
    #[must_use]
    pub fn get_active(&self, active: bool) -> Color {
        if active {
            self.normal
        } else {
            self.inactive
        }
    }

    #[must_use]
    pub fn get_highlight(&self, highlight: bool) -> Color {
        if highlight {
            self.highlight
        } else {
            self.normal
        }
    }
}
