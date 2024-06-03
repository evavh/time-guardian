use std::collections::HashMap;
use std::fs;
use std::time::Duration;

use chrono::Local;
use chrono::NaiveDate;
use color_eyre::Result;
use serde_derive::{Deserialize, Serialize};

use crate::config::Config;

const STATUS_PATH: &str = "/var/lib/time-guardian/status-dev.toml";

#[derive(Serialize, Deserialize)]
pub(crate) struct Counter {
    pub(crate) date: NaiveDate,
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
                eprintln!("Error while loading counter: {err}, resetting");
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

    pub(crate) fn load() -> Result<Self, String> {
        let toml = match fs::read_to_string(STATUS_PATH) {
            Ok(str) => str,
            Err(err) => return Err(err.to_string()),
        };
        let counter: Result<Counter, _> = toml::from_str(&toml);

        match counter {
            Ok(res) => Ok(res),
            Err(err) => Err(format!("{err}")),
        }
    }

    pub(crate) fn store(&self) {
        match crate::store_as_toml(&self, STATUS_PATH) {
            Ok(()) => (),
            Err(err) => eprintln!("Error while trying to store counter: {err}"),
        };
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
