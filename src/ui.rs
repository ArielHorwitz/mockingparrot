use crate::State;
use ratatui::{prelude::Stylize, style::Color, widgets::Paragraph, Frame};

pub fn draw_frame(frame: &mut Frame, state: &mut State) {
    let area = frame.size();
    frame.render_widget(
        Paragraph::new(format!("{:#?}", state))
            .bg(Color::from_hsl(330.0, 100.0, 5.0))
            .fg(Color::Green),
        area,
    );
}
