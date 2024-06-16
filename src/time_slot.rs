use std::time::Duration;

use chrono::NaiveTime;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSecondsWithFrac};

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TimeSlot {
    pub(crate) start: NaiveTime,
    pub(crate) end: NaiveTime,
    #[serde_as(as = "Option<DurationSecondsWithFrac<f64>>")]
    pub(crate) time: Option<Duration>,
}

impl Default for TimeSlot {
    fn default() -> Self {
        let start = NaiveTime::from_hms_opt(0, 0, 0).expect("Valid");
        let end = NaiveTime::from_hms_opt(23, 59, 59).expect("Valid");
        let time = Some(Duration::from_secs(86400));

        Self {
            start,
            end,
            time,
        }
    }
}

impl TimeSlot {
    pub fn contains(&self, time: NaiveTime) -> bool {
        // Not passing midnight
        if self.end >= self.start {
            time >= self.start && time <= self.end
        // Passing midnight
        } else {
            time <= self.start || time >= self.end
        }
    }

    pub fn zero_time(mut self) -> Self {
        self.time = Some(Duration::default());
        self
    }
}

