use anyhow::{Context, Result};
use ratatui::prelude::{Backend, Terminal};

use mockingparrot::app::events;
use mockingparrot::app::state::State;
use mockingparrot::app::ui;

const FRAME_DURATION_MS: u64 = 50;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{e:?}");
    }
}

async fn run() -> Result<()> {
    let state = State::new().context("new app state")?;
    let mut terminal = ratatui::init();
    let app_result = run_app(&mut terminal, state).await;
    ratatui::restore();
    app_result
}

async fn run_app(terminal: &mut Terminal<impl Backend>, mut state: State) -> Result<()> {
    loop {
        terminal
            .draw(|frame| {
                if let Err(e) = ui::draw_frame(frame, &mut state) {
                    todo!("log error: {e}");
                }
            })
            .context("draw frame")?;
        match events::handle(FRAME_DURATION_MS, &mut state)
            .await
            .context("handle events")?
        {
            events::HandleEventResult::None => (),
            events::HandleEventResult::Redraw => terminal.clear().context("clear terminal")?,
            events::HandleEventResult::Quit => return Ok(()),
        };
        state.fix_clamp_ui_selections();
    }
}
