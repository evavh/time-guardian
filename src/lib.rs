use std::thread;
use std::time::Duration;
use std::{collections::HashMap, fs};

use chrono::{Local, NaiveDate};
use serde_derive::{Deserialize, Serialize};

use crate::user_management::{exists, is_active, list_users, logout};

mod notification;
mod user_management;

const STATUS_PATH: &str = "/var/lib/time-guardian/status.toml";

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub short_warning_seconds: usize,
    pub long_warning_seconds: usize,
    pub total_per_day: HashMap<String, usize>,
    // TODO: make this a user setting
}

impl Default for Config {
    fn default() -> Self {
        let total_per_day: HashMap<String, usize> =
            list_users().into_iter().map(|user| (user, 86400)).collect();
        let short_warning_seconds = 30;
        let long_warning_seconds = 300;

        Self {
            total_per_day,
            short_warning_seconds,
            long_warning_seconds,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Counter {
    date: NaiveDate,
    spent_seconds: HashMap<String, usize>,
}

impl Counter {
    fn new(users: &[String]) -> Self {
        Self {
            date: Local::now().date_naive(),
            spent_seconds: initialize_counting(users),
        }
    }

    fn is_outdated(&self) -> bool {
        Local::now().date_naive() != self.date
    }

    fn load() -> Result<Self, String> {
        let toml = match fs::read_to_string(STATUS_PATH) {
            Ok(str) => str,
            Err(err) => return Err(err.to_string()),
        };
        let counter: Result<Counter, _> = toml::from_str(&toml);

        match counter {
            Ok(res) => Ok(res),
            Err(err) => return Err(format!("{}", err)),
        }
    }

    fn store(&self) {
        let toml = toml::to_string(&self).unwrap();
        fs::write(STATUS_PATH, toml).unwrap();
    }
}

pub fn run(config: &Config) -> ! {
    let users: Vec<_> = config
        .total_per_day
        .keys()
        .map(ToString::to_string)
        .collect();

    let mut counter = match Counter::load() {
        Ok(counter) => {
            if !counter.is_outdated() {
                counter
            } else {
                Counter::new(&users)
            }
        }
        Err(err) => {
            dbg!(err);
            Counter::new(&users)
        }
    };

    loop {
        // Reset on new day
        if counter.is_outdated() {
            println!("New day, resetting");
            counter = Counter::new(&users);
        }

        thread::sleep(Duration::from_secs(1));

        for (user, allowed_seconds) in &config.total_per_day {
            if is_active(user) {
                *counter.spent_seconds.get_mut(user).unwrap() += 1;

                if counter.spent_seconds[user] >= *allowed_seconds {
                    logout(user);
                    // To prevent overflows in subtractions
                    continue;
                }

                let seconds_left =
                    allowed_seconds - counter.spent_seconds[user];
                // TODO create if does not exist!
                // TODO change to spent_seconds (not seconds left)

                counter.store();
                fs::write(
                    format!("/var/lib/time-guardian/{user}.status"),
                    format!(
                        "{}\n{}\n",
                        counter.date.format("%d-%m-%Y"),
                        seconds_left
                    ),
                )
                .unwrap();

                // TODO: make short and long warnings different
                // (and multiple possible)
                if seconds_left == config.short_warning_seconds
                    || seconds_left == config.long_warning_seconds
                {
                    notification::notify_user(
                        user,
                        &format!(
                            "You will be logged out in {} seconds!",
                            seconds_left
                        ),
                    );
                }
            }
        }
    }
}

pub fn check_correct(config: &Config) {
    let Config { total_per_day, .. } = config;

    for user in total_per_day.keys() {
        assert!(exists(user), "Error in config: user {user} does not exist");
    }
}

pub(crate) fn initialize_counting(users: &[String]) -> HashMap<String, usize> {
    users
        .iter()
        .map(|user| ((*user).to_string(), 0_usize))
        .collect()
}
