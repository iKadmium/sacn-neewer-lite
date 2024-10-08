use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub struct EventCounter {
    count: u64,
    last_clear: Instant,
    interval: Duration,
    history: VecDeque<u64>,
    max_history_size: usize,
}

impl EventCounter {
    pub fn new(interval: Duration, max_history_size: usize) -> Self {
        EventCounter {
            count: 0,
            last_clear: Instant::now(),
            interval,
            history: VecDeque::with_capacity(max_history_size),
            max_history_size,
        }
    }

    pub fn increment(&mut self) {
        self.count += 1;
    }

    pub fn get_count(&self) -> u64 {
        self.count
    }

    pub fn clear(&mut self) {
        if self.history.len() == self.max_history_size {
            self.history.pop_front();
        }
        self.history.push_back(self.count);
        self.count = 0;
        self.last_clear = Instant::now();
    }

    pub fn time_since_last_clear(&self) -> Duration {
        Instant::now().duration_since(self.last_clear)
    }

    pub fn should_clear(&self) -> bool {
        self.time_since_last_clear() >= self.interval
    }

    pub fn get_history(&self) -> &VecDeque<u64> {
        &self.history
    }
}
