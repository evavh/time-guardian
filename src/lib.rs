use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use std::{collections::HashMap, fs};

use chrono::{Local, NaiveDate};
use color_eyre::Result;
use serde_derive::{Deserialize, Serialize};

use crate::user_management::{is_active, logout};
use crate::config::Config;

mod config;
mod notification;
mod user_management;

const CONFIG_PATH: &str = "/etc/time-guardian/config-dev.toml";
const PREV_CONFIG_PATH: &str = "/etc/time-guardian/prev-config-dev.toml";
const FALLBACK_CONFIG_PATH: &str =
    "/etc/time-guardian/fallback-config-dev.toml";
const TEMPLATE_CONFIG_PATH: &str =
    "/etc/time-guardian/template-config-dev.toml";
const STATUS_PATH: &str = "/var/lib/time-guardian/status-dev.toml";

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
    if match Config::load(TEMPLATE_CONFIG_PATH) {
        Ok(config) => config != config::Config::default(),
        Err(_) => true,
    } {
        println!("Writing new template config");
        Config::default().store(TEMPLATE_CONFIG_PATH);
    }

    let mut config = match Config::load(CONFIG_PATH) {
        Ok(config) => config,
        Err(err) => {
            eprintln!(
                "Error while initially loading config, using previous config\nCause: {err:?}"
            );
            match Config::load(PREV_CONFIG_PATH) {
                Ok(config) => config,
                Err(err) => {
                    eprintln!("Error while loading previous config on startup, using fallback\nCause: {err:?}");
                    Config::load(FALLBACK_CONFIG_PATH).unwrap()
                }
            }
        }
    };

    let users: Vec<_> = config.users().map(ToString::to_string).collect();

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
            config = match Config::load(CONFIG_PATH) {
                Ok(new_config) => {
                    Config::store(&new_config, PREV_CONFIG_PATH);
                    new_config
                }
                Err(err) => {
                    eprintln!("Error loading config: {err:?}");
                    old_config
                }
            }
        }

        thread::sleep(Duration::from_secs(1));

        for (user, config) in config.iter() {
            if is_active(user) {
                *counter.spent_seconds.get_mut(user).expect("Initialized from the hashmap we iterate over, should be in there") += 1;
                println!(
                    "{user} spent {} out of {}",
                    counter.spent_seconds[user], config.allowed_seconds
                );

                if counter.spent_seconds[user] >= config.allowed_seconds {
                    logout(user);
                    // This user doesn't need to be accounted for right now
                    continue;
                }

                let seconds_left = config
                    .allowed_seconds
                    .saturating_sub(counter.spent_seconds[user]);

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

pub(crate) fn initialize_counting(users: &[String]) -> HashMap<String, usize> {
    users
        .iter()
        .map(|user| ((*user).to_string(), 0_usize))
        .collect()
}
