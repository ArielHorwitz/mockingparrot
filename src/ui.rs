use crate::state::State;
use anyhow::{Context, Result};
use ratatui::{
    prelude::{Constraint, Direction, Layout, Rect, Style, Stylize},
    style::Color,
    widgets::{Block, List, ListState, Paragraph, Wrap},
    Frame,
};
use tui_textarea::TextArea;

const STATUSBAR_HELP_TEXT: &str = "Ctrl+q - Quit, F1 - conversation, F2 - config/debug";

pub struct UiState<'a> {
    pub tab: ViewTab,
    pub status_bar_text: String,
    pub textarea: TextArea<'a>,
    pub feedback: String,
    pub system_instruction_selection: ListState,
    pub debug: bool,
    pub key_event_debug: String,
}

impl<'a> Default for UiState<'a> {
    fn default() -> Self {
        Self {
            tab: ViewTab::Conversation,
            status_bar_text: format!("Welcome to {}", crate::APP_TITLE),
            textarea: get_textarea(),
            feedback: "Debug logs empty.".to_owned(),
            system_instruction_selection: ListState::default().with_selected(Some(0)),
            debug: false,
            key_event_debug: Default::default(),
        }
    }
}

impl<'a> std::fmt::Debug for UiState<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Debug mode: {:?}\n{}", self.debug, self.key_event_debug)
    }
}

pub fn get_textarea() -> TextArea<'static> {
    let mut textarea = TextArea::default();
    textarea.set_style(Style::new().bg(Color::Rgb(0, 25, 25)).fg(Color::White));
    textarea.set_line_number_style(Style::new().bg(Color::Black).fg(Color::Cyan));
    textarea.set_cursor_style(Style::new().bg(Color::Rgb(200, 200, 200)));
    textarea
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ViewTab {
    Conversation,
    NewConversation,
    Config,
}

impl ViewTab {
    pub fn next_tab(self) -> ViewTab {
        match self {
            ViewTab::Conversation => ViewTab::NewConversation,
            ViewTab::NewConversation => ViewTab::Config,
            ViewTab::Config => ViewTab::Conversation,
        }
    }
}

pub fn draw_ui_frame(frame: &mut Frame, state: &State, ui_state: &UiState) -> Result<()> {
    let layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ],
    )
    .split(frame.size());

    // Title bar
    frame.render_widget(
        Block::new()
            .title(crate::APP_TITLE_FULL)
            .bg(Color::Blue)
            .fg(Color::LightGreen)
            .bold(),
        *layout.first().context("ui index")?,
    );

    // Status bar
    let now = {
        let now = chrono::Local::now();
        if ui_state.debug {
            format!("{}", now.format("%Y-%m-%d %H:%M:%S%.3f"))
        } else {
            format!("{}", now.format("%Y-%m-%d %H:%M:%S"))
        }
    };
    frame.render_widget(
        Paragraph::new(ui_state.status_bar_text.as_str())
            .bg(Color::DarkGray)
            .fg(Color::LightGreen),
        *layout.get(2).context("ui index")?,
    );
    frame.render_widget(
        Paragraph::new(format!("{STATUSBAR_HELP_TEXT} | {now}"))
            .bg(Color::Black)
            .fg(Color::Green),
        *layout.get(3).context("ui index")?,
    );

    // Main UI
    match ui_state.tab {
        ViewTab::Conversation => {
            draw_conversation(frame, *layout.get(1).context("ui index")?, state, ui_state)?
        }
        ViewTab::NewConversation => {
            new_conversation(frame, *layout.get(1).context("ui index")?, state, ui_state)?;
        }
        ViewTab::Config => {
            draw_config(frame, *layout.get(1).context("ui index")?, state, ui_state)?
        }
    };
    Ok(())
}

pub fn draw_conversation(
    frame: &mut Frame,
    rect: Rect,
    state: &State,
    ui_state: &UiState,
) -> Result<()> {
    let layout = Layout::new(
        Direction::Vertical,
        [Constraint::Fill(1), Constraint::Length(10)],
    )
    .split(rect);

    // Conversation display
    let convo = if state.config.api.key.is_empty() {
        "MISSING API KEY!\n\nEnter an API key in your `config.toml` file to start chatting...\n\nhttps://platform.openai.com".to_owned()
    } else {
        state.conversation.to_string()
    };
    frame.render_widget(
        Paragraph::new(convo)
            .wrap(Wrap { trim: false })
            .bg(Color::Rgb(0, 0, 35))
            .fg(Color::Rgb(0, 255, 0)),
        *layout.first().context("ui index")?,
    );

    // Text input
    frame.render_widget(
        ui_state.textarea.widget(),
        *layout.get(1).context("ui index")?,
    );
    Ok(())
}

fn new_conversation(
    frame: &mut Frame,
    rect: Rect,
    state: &State,
    ui_state: &UiState,
) -> Result<()> {
    let list_items = state
        .config
        .system
        .instructions
        .iter()
        .map(|i| format!(">> {}\n{}", i.name, i.message));
    let list = List::new(list_items).highlight_style(Color::LightGreen);
    let mut list_state = ui_state.system_instruction_selection.clone();
    frame.render_stateful_widget(list, rect, &mut list_state);
    Ok(())
}

pub fn draw_config(frame: &mut Frame, rect: Rect, state: &State, ui_state: &UiState) -> Result<()> {
    let main_layout = Layout::new(
        Direction::Vertical,
        [Constraint::Fill(1), Constraint::Fill(1)],
    )
    .split(rect);
    let top_layout = Layout::new(
        Direction::Horizontal,
        [Constraint::Fill(1), Constraint::Fill(1)],
    )
    .split(*main_layout.first().context("ui index")?);

    frame.render_widget(
        Paragraph::new(format!("{:#?}", state.config.chat))
            .wrap(Wrap { trim: false })
            .bg(Color::Rgb(0, 20, 35))
            .fg(Color::Rgb(125, 150, 0)),
        *top_layout.first().context("ui index")?,
    );
    frame.render_widget(
        Paragraph::new(format!("{:#?}", ui_state))
            .wrap(Wrap { trim: false })
            .bg(Color::Rgb(30, 0, 35))
            .fg(Color::Rgb(125, 150, 0)),
        *top_layout.get(1).context("ui index")?,
    );
    frame.render_widget(
        Paragraph::new(ui_state.feedback.as_str())
            .wrap(Wrap { trim: false })
            .bg(Color::Rgb(30, 30, 0))
            .fg(Color::Rgb(125, 150, 0)),
        *main_layout.get(1).context("ui index")?,
    );
    Ok(())
}
