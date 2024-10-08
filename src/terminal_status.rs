use std::time::Duration;

use ratatui::style::Color;

use crate::event_counter::EventCounter;

pub struct TerminalStatus {
    pub status: String,
    pub color: Color,
    pub event_counter: EventCounter,
}

impl TerminalStatus {
    pub fn new() -> Self {
        Self {
            status: String::new(),
            color: Color::Reset,
            event_counter: EventCounter::new(Duration::from_secs(1), 20),
        }
    }
}
