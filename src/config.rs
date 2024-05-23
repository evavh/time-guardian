use crate::user_management::{exists, list_users};

use chrono::NaiveDate;
use color_eyre::{eyre::Context, Result};
use serde_derive::{Deserialize, Serialize};

use std::{collections::HashMap, fs};

const CONFIG_PATH: &str = "/etc/time-guardian/config-dev.toml";
const PREV_CONFIG_PATH: &str = "/etc/time-guardian/prev-config-dev.toml";
const FALLBACK_CONFIG_PATH: &str =
    "/etc/time-guardian/fallback-config-dev.toml";
const TEMPLATE_CONFIG_PATH: &str =
    "/etc/time-guardian/template-config-dev.toml";
const STATUS_PATH: &str = "/var/lib/time-guardian/status-dev.toml";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("User {0} doesn't exist")]
    UserDoesntExist(String),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Config(HashMap<String, UserConfig>);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[allow(clippy::module_name_repetitions)]
pub struct UserConfig {
    // TODO: make warnings a user-editable setting
    pub short_warning_seconds: usize,
    pub long_warning_seconds: usize,
    pub allowed_seconds: usize,
    pub rampup: Option<Rampup>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Rampup {
    pub speed: Speed,
    pub start_date: NaiveDate,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Speed {
    ConstantSeconds(isize),
    Percentage(f32),
}

impl Default for Config {
    fn default() -> Self {
        let users = match list_users() {
            Ok(users) => users,
            Err(err) => {
                eprintln!("Couldn't list users in home: {err:?}");
                vec!["example_user".to_owned()]
            }
        };

        let rampup = Rampup {
            speed: Speed::ConstantSeconds(1),
            start_date: NaiveDate::from_ymd_opt(2024, 5, 1)
                .expect("Date exists"),
        };
        let user_config = UserConfig {
            short_warning_seconds: 30,
            long_warning_seconds: 300,
            allowed_seconds: 86400,
            rampup: Some(rampup),
        };

        let per_user: HashMap<String, UserConfig> = users
            .into_iter()
            .map(|user| (user, user_config.clone()))
            .collect();

        Self(per_user)
    }
}

impl Config {
    pub(crate) fn initialize_from_files() -> Self {
        if match Config::load(TEMPLATE_CONFIG_PATH) {
            Ok(config) => config != config::Config::default(),
            Err(_) => true,
        } {
            println!("Writing new template config");
            Config::default().store(TEMPLATE_CONFIG_PATH);
        }

        match Config::load(CONFIG_PATH) {
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
        }
    }

    pub(crate) fn reload(&mut self) {
        let old_config = self.clone();

        *self = match Config::load(CONFIG_PATH) {
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

    pub fn load(path: &str) -> Result<Self> {
        let data = fs::read_to_string(path)
            .wrap_err(format!("Couldn't read config from file {path}"))?;
        let new_config: Self = toml::from_str(&data)
            .wrap_err(format!("Couldn't deserialize file {path}"))?;

        new_config
            .check_correct()
            .wrap_err("New config has errors")?;

        Ok(new_config)
    }

    pub fn store(&self, path: &str) {
        let serialized = match toml::to_string(&self) {
            Ok(res) => res,
            Err(err) => {
                eprintln!("Couldn't serialize config for path {path}: {err}");
                return;
            }
        };
        match fs::write(path, serialized) {
            Ok(()) => (),
            Err(err) => {
                eprintln!(
                    "Couldn't store config to disk for path {path}: {err}"
                );
            }
        }
    }

    pub fn check_correct(&self) -> Result<(), Error> {
        for user in self.users() {
            if !exists(&user) {
                return Err(Error::UserDoesntExist(user.clone()));
            };
        }

        Ok(())
    }

    pub fn users(&self) -> impl Iterator<Item = String> + '_ {
        self.0.keys().map(ToString::to_string)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &UserConfig)> + '_ {
        self.0.iter()
    }
}
