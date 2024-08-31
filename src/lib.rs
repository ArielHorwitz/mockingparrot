pub mod api;
pub mod config;
pub mod conversation;
pub mod events;
pub mod hotkeys;
pub mod state;
pub mod ui;

const APP_TITLE: &str = "MockingParrot";
const APP_TITLE_FULL: &str = "MockingParrot AI Chat Client";

fn get_timestamp() -> String {
    format!("{}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"))
}
