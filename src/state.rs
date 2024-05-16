use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct State {
    pub feedback: String,
    pub frame_count: u64,
    pub last_frame: Instant,
    pub frame_time: Duration,
}

impl Default for State {
    fn default() -> Self {
        Self {
            feedback: Default::default(),
            frame_count: Default::default(),
            last_frame: Instant::now(),
            frame_time: Default::default(),
        }
    }
}
