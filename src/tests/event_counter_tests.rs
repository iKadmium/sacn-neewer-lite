#[cfg(test)]
mod tests {
    use crate::event_counter::EventCounter;

    use std::{thread::sleep, time::Duration};

    #[test]
    fn test_initialization() {
        let interval = Duration::from_secs(5);
        let max_history_size = 20;
        let counter = EventCounter::new(interval, max_history_size);
        assert_eq!(counter.get_count(), 0);
        assert!(counter.time_since_last_clear() < interval);
    }

    #[test]
    fn test_increment() {
        let interval = Duration::from_secs(5);
        let max_history_size = 20;
        let mut counter = EventCounter::new(interval, max_history_size);
        counter.increment();
        assert_eq!(counter.get_count(), 1);
        counter.increment();
        assert_eq!(counter.get_count(), 2);
    }

    #[test]
    fn test_clear() {
        let interval = Duration::from_secs(5);
        let max_history_size = 20;
        let mut counter = EventCounter::new(interval, max_history_size);
        counter.increment();
        counter.increment();
        assert_eq!(counter.get_count(), 2);
        counter.clear();
        assert_eq!(counter.get_count(), 0);
        assert!(counter.time_since_last_clear() < interval);
    }

    #[test]
    fn test_time_since_last_clear() {
        let interval = Duration::from_secs(1);
        let max_history_size = 20;
        let counter = EventCounter::new(interval, max_history_size);
        sleep(Duration::from_secs(2));
        assert!(counter.time_since_last_clear() >= Duration::from_secs(2));
    }

    #[test]
    fn test_should_clear() {
        let interval = Duration::from_secs(1);
        let max_history_size = 20;
        let counter = EventCounter::new(interval, max_history_size);
        assert!(!counter.should_clear());
        sleep(Duration::from_secs(2));
        assert!(counter.should_clear());
    }

    #[test]
    fn test_clear_resets_timer() {
        let interval = Duration::from_secs(1);
        let max_history_size = 20;
        let mut counter = EventCounter::new(interval, max_history_size);
        sleep(Duration::from_secs(2));
        assert!(counter.should_clear());
        counter.clear();
        assert!(!counter.should_clear());
    }
}
