use std::time::{Duration, SystemTime};

use tracing::{error, warn};

use crate::dmg::ClockTicks;

#[derive(Debug)]
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
        if let Err(err) = elapsed {
            error!("{}", err.to_string());
        } else {
            let elapsed = elapsed.unwrap();
            if elapsed > self.period_time {
                // warn!("Last iteration took longer than 1 clock tick !");
                self.last_stamp = SystemTime::now();
                return;
            }

            let to_sleep = self.period_time - elapsed;
            std::thread::sleep(to_sleep);
            self.last_stamp = SystemTime::now();
        }
    }
}

pub struct TickCoordinator {
    ticks_to_wait: isize,
}

impl TickCoordinator {
    pub fn new() -> Self {
        Self { ticks_to_wait: 1 }
    }

    pub fn tick(&mut self) -> bool {
        self.ticks_to_wait -= 1;
        self.ticks_to_wait <= 0
    }

    pub fn ticks(&mut self, ticks: ClockTicks) -> bool {
        self.ticks_to_wait -= ticks as isize;
        self.ticks_to_wait <= 0
    }

    pub fn tick_all(&mut self) -> ClockTicks {
        if self.ticks_to_wait <= 0 {
            return 0;
        }
        let res = self.ticks_to_wait as ClockTicks;
        self.ticks_to_wait = 0;
        res
    }

    pub fn wait_for(&mut self, ticks: ClockTicks) {
        self.ticks_to_wait = ticks as isize;
    }
}
