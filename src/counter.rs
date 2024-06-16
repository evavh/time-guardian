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
use crate::config::User;
use crate::file_io;
use crate::logging::log_error;
use crate::time_slot::TimeSlot;

#[derive(Serialize, Deserialize)]
pub(crate) struct Counter {
    pub(crate) date: NaiveDate,
    pub(crate) counter: HashMap<User, UserCounter>,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct UserCounter {
    #[serde_as(as = "DurationSecondsWithFrac<f64>")]
    pub(crate) total_spent: Duration,
    pub(crate) time_slots: Vec<TimeSlot>,
}

impl Counter {
    pub(crate) fn initialize(config: &Config) -> Counter {
        let counter = match Counter::load() {
            Ok(counter) => {
                if counter.is_outdated() {
                    Counter::new(&config)
                } else {
                    counter
                }
            }
            Err(err) => {
                error!("Error while loading counter: {err}, resetting");
                Counter::new(&config)
            }
        };

        counter.store();
        counter
    }

    pub(crate) fn new(config: &Config) -> Self {
        let users = config
            .iter()
            .map(|(user, user_config)| {
                let time_slots = user_config
                    .time_slots
                    .clone()
                    .into_iter()
                    .map(|ts| ts.zero_time())
                    .collect();
                let user_counter = UserCounter {
                    total_spent: Duration::default(),
                    time_slots,
                };
                (user.clone(), user_counter)
            })
            .collect();

        Self {
            date: Local::now().date_naive(),
            counter: users,
        }
    }

    pub(crate) fn is_outdated(&self) -> bool {
        Local::now().date_naive() != self.date
    }

    pub(crate) fn load() -> Result<Self> {
        let file_content = fs::read_to_string(file_io::path::STATUS)?;
        let counter: Result<Counter, _> = file_io::from_str(&file_content);

        Ok(counter?)
    }

    pub(crate) fn store(&self) {
        log_error(
            file_io::store(&self, file_io::path::STATUS),
            "Error while trying to store counter",
        );
    }

    pub(crate) fn add(mut self, user: &str, duration: Duration) -> Self {
        let user_counter = self.counter.get_mut(user)
            .expect("Initialized from the hashmap, should be in there");

        user_counter.total_spent += duration;
        self
    }
}
