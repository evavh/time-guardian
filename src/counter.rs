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

#[serde_as]
#[derive(Serialize, Deserialize)]
pub(crate) struct Counter {
    pub(crate) date: NaiveDate,
    #[serde_as(as = "HashMap<_, DurationSecondsWithFrac<f64>>")]
    pub(crate) spent: HashMap<String, Duration>,
}

impl Counter {
    pub(crate) fn initialize(config: &Config) -> Counter {
        let counter = match Counter::load() {
            Ok(counter) => {
                if counter.is_outdated() {
                    Counter::new(config.users())
                } else {
                    counter
                }
            }
            Err(err) => {
                error!("Error while loading counter: {err}, resetting");
                Counter::new(config.users())
            }
        };

        counter.store();
        counter
    }

    pub(crate) fn new(users: impl Iterator<Item = String>) -> Self {
        let spent = users.map(|user| (user, Duration::default())).collect();

        Self {
            date: Local::now().date_naive(),
            spent,
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
        let count = self
            .spent
            .get_mut(user)
            .expect("Initialized from the hashmap, should be in there");

        *count += duration;
        self
    }
}
