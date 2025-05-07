/**
 * @file clock/tick.rs
 * @author Nguyen Le Duy
 * @date 03/01/2025
 * @brief Tick handling module
 */
use std::time::Duration;

use crate::common::MHZ;

#[derive(Clone, Debug)]
pub enum Ticks {
    Duration(Duration),
    Exact(u64),
}

impl Ticks {
    pub const _1MHZ: Self = Ticks::Exact(150);
    pub const CKL_SYS: Self = Ticks::Exact(1);

    pub fn into_ticks_number(self) -> u64 {
        match self {
            Ticks::Duration(dur) => {
                let base = 1.0 / (150 * MHZ) as f64;
                let base = Duration::from_secs_f64(base);
                dur.div_duration_f64(base).ceil() as u64
            }
            Ticks::Exact(tick) => tick,
        }
    }
}

impl From<u64> for Ticks {
    fn from(tick: u64) -> Self {
        Self::Exact(tick)
    }
}

impl From<Duration> for Ticks {
    fn from(dur: Duration) -> Self {
        Self::Duration(dur)
    }
}
