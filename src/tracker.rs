use std::collections::HashMap;
use std::fs;
use std::time::Duration;

use chrono::Local;
use chrono::NaiveDate;
use color_eyre::Result;
use log::error;
use serde_derive::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSecondsWithFrac};

use crate::config::Config;
use crate::file_io;
use crate::logging::log_error;
use crate::time_slot::TimeSlot;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Tracker {
    pub(crate) date: NaiveDate,
    pub(crate) counter: HashMap<String, UserCounter>,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct UserCounter {
    #[serde_as(as = "DurationSecondsWithFrac<f64>")]
    pub(crate) total_spent: Duration,
    pub(crate) time_slots: Option<Vec<TimeSlot>>,
}

impl UserCounter {
    pub fn add_to_total_spent(&mut self, duration: Duration) {
        self.total_spent += duration;
    }

    pub fn add_to_current_timeslots(&mut self, duration: Duration) {
        self.time_slots = match &mut self.time_slots {
            Some(ref mut time_slots) => {
                for slot in time_slots.iter_mut() {
                    if slot.contains(Local::now().naive_local().time()) {
                        slot.time = slot.time.map(|t| t + duration);
                    }
                }
                Some(time_slots.clone())
            }
            None => None,
        };
    }
}

impl Tracker {
    pub(crate) fn initialize(config: &Config) -> Tracker {
        let tracker = match Tracker::load() {
            Ok(tracker) => {
                if tracker.is_outdated() {
                    Tracker::new(config)
                } else {
                    tracker
                }
            }
            Err(err) => {
                error!("Error while loading tracker: {err}, resetting");
                Tracker::new(config)
            }
        };

        tracker.store();
        tracker
    }

    pub(crate) fn new(config: &Config) -> Self {
        let counter = config
            .iter()
            .map(|(user, user_config)| {
                let time_slots = user_config
                    .time_slots
                    .clone()
                    .map(|x| x.into_iter().map(TimeSlot::zero_time).collect());
                let user_counter = UserCounter {
                    total_spent: Duration::default(),
                    time_slots,
                };
                (user.clone(), user_counter)
            })
            .collect();

        Self {
            date: Local::now().date_naive(),
            counter,
        }
    }

    pub(crate) fn is_outdated(&self) -> bool {
        Local::now().date_naive() != self.date
    }

    pub(crate) fn load() -> Result<Self> {
        let file_content = fs::read_to_string(file_io::path::STATUS)?;
        let tracker: Result<Tracker, _> = file_io::from_str(&file_content);

        tracker
    }

    pub(crate) fn store(&self) {
        log_error(
            file_io::store(&self, file_io::path::STATUS),
            "Error while trying to store tracker",
        );
    }

    pub(crate) fn add(&mut self, user: &str, duration: Duration) {
        let user_counter = self
            .counter
            .get_mut(user)
            .expect("Initialized from the hashmap, should be in there");

        user_counter.add_to_total_spent(duration);
        user_counter.add_to_current_timeslots(duration);
    }

    pub(crate) fn timeslot_over_time(
        &self,
        config: &Config,
        user: &str,
    ) -> bool {
        let Some(allowed_timeslots) = &config.user(user).time_slots else {
            return false;
        };

        let Some(spent_timeslots) = &self.counter[user].time_slots else {
            return false;
        };

        for allowed_timeslot in allowed_timeslots {
            for spent_timeslot in spent_timeslots {
                if allowed_timeslot == spent_timeslot
                    && spent_timeslot.time >= allowed_timeslot.time
                {
                    return true;
                };
            }
        }

        false
    }
}
