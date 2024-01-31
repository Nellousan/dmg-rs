use std::time::{Duration, SystemTime};

use tracing::{error, warn};

pub struct Clock {
    last_stamp: SystemTime,
    period_time: Duration,
}

impl Clock {
    pub fn new() -> Self {
        Self {
            last_stamp: SystemTime::now(),
            period_time: Duration::from_nanos(238u64),
        }
    }

    pub fn tick(&mut self) {
        let elapsed = self.last_stamp.elapsed();
        if let Err(err) = elapsed.clone() {
            error!("{}", err.to_string());
        }
        let elapsed = elapsed.unwrap();
        if elapsed > self.period_time {
            warn!("Last iteration took longer than 1 clock tick !");
            return;
        }

        let to_sleep = self.period_time - elapsed;
        std::thread::sleep(to_sleep);
        self.last_stamp = SystemTime::now();
    }
}

pub struct TickCoordinator {
    ticks_to_wait: usize,
}

impl TickCoordinator {
    pub fn new() -> Self {
        Self { ticks_to_wait: 1 }
    }

    pub fn tick(&mut self) -> bool {
        self.ticks_to_wait -= 1;
        self.ticks_to_wait <= 0
    }

    pub fn wait_for(&mut self, ticks: usize) {
        self.ticks_to_wait = ticks;
    }
}
