use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

pub struct ThreadSafeCounter {
    count: Mutex<i32>,
}

impl ThreadSafeCounter {
    pub fn new() -> Self {
        Self {
            count: Mutex::new(0),
        }
    }

    pub fn increment_and_get(&self) -> i32 {
        let mut count = self.count.lock().unwrap();
        *count += 1;
        *count
    }

    pub fn reset(&self) {
        *self.count.lock().unwrap() = 0;
    }
}

pub static COUNTER: Lazy<Arc<ThreadSafeCounter>> =
    Lazy::new(|| Arc::new(ThreadSafeCounter::new()));