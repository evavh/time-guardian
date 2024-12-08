use std::time::Duration;

use jiff::{civil::Time, Zoned};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSecondsWithFrac};

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeSlot {
    pub(crate) start: Time,
    pub(crate) end: Time,
    #[serde_as(as = "Option<DurationSecondsWithFrac<f64>>")]
    pub(crate) time: Option<Duration>,
}

impl Default for TimeSlot {
    fn default() -> Self {
        let start = Time::MIN;
        let end = Time::MAX;
        let time = Some(Duration::from_secs(86400));

        Self {
            start,
            end,
            time,
        }
    }
}

// Only compare start and end, not spent/allowed time
impl PartialEq for TimeSlot {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end
    }
}

impl Eq for TimeSlot {}

impl TimeSlot {
    pub fn contains(&self, time: Zoned) -> bool {
        let time = time.datetime().time();
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

