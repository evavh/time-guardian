use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use std::{collections::HashMap, fs};

use chrono::{Local, NaiveDate};
use color_eyre::eyre::Context;
use color_eyre::Result;
use serde_derive::{Deserialize, Serialize};
use thiserror::Error;

use crate::user_management::{exists, is_active, list_users, logout};

mod notification;
mod user_management;

const CONFIG_PATH: &str = "/etc/time-guardian/config.toml";
const PREV_CONFIG_PATH: &str = "/etc/time-guardian/prev-config.toml";
const FALLBACK_CONFIG_PATH: &str = "/etc/time-guardian/fallback-config.toml";
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
        let users = match list_users() {
            Ok(users) => users,
            Err(err) => {
                eprintln!("Couldn't list users in home: {err:?}");
                vec![]
            }
        };

        let total_per_day: HashMap<String, usize> =
            users.into_iter().map(|user| (user, 86400)).collect();
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

pub fn run() -> ! {
    let mut config = match load_config(CONFIG_PATH) {
        Ok(config) => config,
        Err(err) => {
            eprintln!(
                "Error while initially loading config, using previous config\nCause: {err:?}"
            );
            match load_config(PREV_CONFIG_PATH) {
                Ok(config) => config,
                Err(err) => {
                    eprintln!("Error while loading previous config on startup, using fallback\nCause: {err:?}");
                    dbg!(load_config(FALLBACK_CONFIG_PATH).unwrap())
                }
            }
        }
    };

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
            eprintln!("Error while loading counter: {err}, resetting");
            Counter::new(&users)
        }
    };

    match counter.store() {
        Ok(()) => (),
        Err(err) => eprintln!("Error while trying to store counter: {err}"),
    };

    loop {
        // Reset on new day
        if counter.is_outdated() {
            println!("New day, resetting");
            counter = Counter::new(&users);

            let old_config = config;
            config = match load_config(CONFIG_PATH) {
                Ok(new_config) => {
                    store_config(&new_config, PREV_CONFIG_PATH);
                    new_config
                }
                Err(err) => {
                    eprintln!("Error loading config: {err:?}");
                    old_config
                }
            }
        }

        thread::sleep(Duration::from_secs(1));

        for (user, allowed_seconds) in &config.total_per_day {
            if is_active(user) {
                *counter.spent_seconds.get_mut(user).expect("Initialized from the hashmap we iterate over, should be in there") += 1;

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

fn store_config(config: &Config, path: &str) {
    let serialized = match toml::to_string(config) {
        Ok(res) => res,
        Err(err) => {
            eprintln!("Couldn't serialize config for path {path}: {err}");
            return;
        }
    };
    match fs::write(path, serialized) {
        Ok(_) => (),
        Err(err) => {
            eprintln!("Couldn't store config to disk for path {path}: {err}")
        }
    }
}

fn load_config(path: &str) -> Result<Config> {
    let data = fs::read_to_string(path)
        .wrap_err(format!("Couldn't read config from file {path}"))?;
    let new_config: Config = toml::from_str(&data)
        .wrap_err(format!("Couldn't deserialize file {path}"))?;

    check_correct(&new_config).wrap_err(format!("New config has errors"))?;

    Ok(new_config)
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("User {0} doesn't exist")]
    UserDoesntExist(String),
}

pub fn check_correct(config: &Config) -> Result<(), Error> {
    let Config { total_per_day, .. } = config;

    for user in total_per_day.keys() {
        if !exists(user) {
            return Err(Error::UserDoesntExist(user.to_owned()));
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
