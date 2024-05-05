use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use std::{collections::HashMap, fs};

use chrono::{Local, NaiveDate};
use serde_derive::{Deserialize, Serialize};

use crate::user_management::{exists, is_active, list_users, logout};

mod notification;
mod user_management;

pub const CONFIG_PATH: &str = "/etc/time-guardian/config.toml";
const STATUS_PATH: &str = "/var/lib/time-guardian/status.toml";

#[derive(Debug, Serialize, Deserialize)]
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
            short_warning_seconds,
            long_warning_seconds,
            total_per_day,
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
            Err(err) => Err(format!("{err}")),
        }
    }

    fn store(&self) -> Result<(), std::io::Error> {
        let toml = toml::to_string(&self)
            .expect("Serializing failed, probably an error in toml");

        if !PathBuf::from(STATUS_PATH)
            .parent()
            .expect("This path should have a parent")
            .exists()
        {
            std::fs::create_dir_all(
                PathBuf::from(STATUS_PATH)
                    .parent()
                    .expect("This path should have a parent"),
            )?;
        }
        fs::write(STATUS_PATH, toml)?;
        Ok(())
    }
}

pub fn run(mut config: Config) -> ! {
    let users: Vec<_> = config
        .total_per_day
        .keys()
        .map(ToString::to_string)
        .collect();

    let mut counter = match Counter::load() {
        Ok(counter) => {
            if counter.is_outdated() {
                Counter::new(&users)
            } else {
                counter
            }
        }
        Err(err) => {
            dbg!(err);
            Counter::new(&users)
        }
    };

    match counter.store() {
        Ok(()) => (),
        Err(e) => panic!("Error while trying to store counter: {e}"),
    };

    loop {
        // Reset on new day
        if counter.is_outdated() {
            println!("New day, resetting");
            counter = Counter::new(&users);

            let old_config = config;
            config = match confy::load_path(CONFIG_PATH) {
                Ok(new_config) => match check_correct(&new_config) {
                    Ok(()) => new_config,
                    Err(e) => {
                        println!(
                            "New config has errors ({e}), using old config"
                        );
                        old_config
                    }
                },

                Err(e) => {
                    eprintln!("Couldn't load config, error: {e}");
                    old_config
                }
            };
        }

        thread::sleep(Duration::from_secs(1));

        for (user, allowed_seconds) in &config.total_per_day {
            if is_active(user) {
                *counter.spent_seconds.get_mut(user).unwrap() += 1;

                if counter.spent_seconds[user] >= *allowed_seconds {
                    logout(user);
                    // This user doesn't need to be accounted for right now
                    continue;
                }

                let seconds_left =
                    allowed_seconds.saturating_sub(counter.spent_seconds[user]);

                counter
                    .store()
                    .expect("This worked before starting, and should work now");

                // TODO: make short and long warnings different
                // (and multiple possible)
                if seconds_left == config.short_warning_seconds
                    || seconds_left == config.long_warning_seconds
                {
                    notification::notify_user(
                        user,
                        &format!(
                            "You will be logged out in {seconds_left} seconds!",
                        ),
                    );
                }
            }
        }
    }
}

pub fn check_correct(config: &Config) -> Result<(), String> {
    let Config { total_per_day, .. } = config;

    for user in total_per_day.keys() {
        if !exists(user) {
            return Err(format!("Error in config: user {user} does not exist"));
        };
    }

    Ok(())
}

pub(crate) fn initialize_counting(users: &[String]) -> HashMap<String, usize> {
    users
        .iter()
        .map(|user| ((*user).to_string(), 0_usize))
        .collect()
}
