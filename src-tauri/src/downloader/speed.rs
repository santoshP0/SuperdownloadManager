use std::collections::VecDeque;
use std::time::Instant;

pub struct SpeedTracker {
    samples: VecDeque<(Instant, u64)>,
    window_secs: f64,
}

impl SpeedTracker {
    pub fn new() -> Self {
        Self {
            samples: VecDeque::new(),
            window_secs: 3.0,
        }
    }

    /// Feed the current total downloaded bytes; returns bytes/sec.
    pub fn update(&mut self, total_bytes: u64) -> f64 {
        let now = Instant::now();
        self.samples.push_back((now, total_bytes));

        // Evict samples outside the rolling window
        while let Some(front) = self.samples.front() {
            if now.duration_since(front.0).as_secs_f64() > self.window_secs {
                self.samples.pop_front();
            } else {
                break;
            }
        }

        if self.samples.len() < 2 {
            return 0.0;
        }

        let oldest = self.samples.front().unwrap();
        let newest = self.samples.back().unwrap();
        let elapsed = newest.0.duration_since(oldest.0).as_secs_f64();

        if elapsed < 0.01 {
            return 0.0;
        }

        newest.1.saturating_sub(oldest.1) as f64 / elapsed
    }
}
