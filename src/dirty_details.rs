use tokio::time::Instant;

pub struct DirtyDetails {
    dirty: bool,
    last_clean_time: Instant,
}

impl DirtyDetails {
    pub fn new() -> Self {
        Self {
            dirty: true,
            last_clean_time: Instant::now(),
        }
    }

    pub fn clean(&mut self) {
        self.dirty = false;
        self.last_clean_time = Instant::now();
    }

    pub fn is_dirty(&self) -> bool {
        return self.dirty || self.last_clean_time.elapsed().as_secs() > 10;
    }

    pub fn dirty(&mut self) {
        self.dirty = true;
    }
}
