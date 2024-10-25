pub mod api;
pub mod app;
pub mod chat;
pub mod config;

const APP_TITLE: &str = "MockingParrot";
const APP_TITLE_FULL: &str = "MockingParrot AI Chat Client";

fn get_timestamp() -> String {
    format!("{}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"))
}
