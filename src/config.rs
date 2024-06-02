use crate::user_management::{exists, list_users};

use chrono::{Local, NaiveDate};
use color_eyre::{eyre::Context, Result};
use serde_derive::{Deserialize, Serialize};

use std::{collections::HashMap, fs};

const CONFIG_PATH: &str = "/etc/time-guardian/config.toml";
const PREV_CONFIG_PATH: &str = "/etc/time-guardian/prev-config.toml";
const FALLBACK_CONFIG_PATH: &str = "/etc/time-guardian/fallback-config.toml";
const TEMPLATE_CONFIG_PATH: &str = "/etc/time-guardian/template-config.toml";
const RAMPEDUP_PATH: &str = "/var/lib/time-guardian/rampedup.toml";

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
    pub short_warning_seconds: u32,
    pub long_warning_seconds: u32,
    pub allowed_seconds: u32,
    pub rampup: Option<Rampup>,
}

impl UserConfig {
    pub fn clamp_rampup(mut self) -> Self {
        self.rampup = self.rampup.map(Rampup::clamp_percentage);
        self
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Rampup {
    pub speed: Speed,
    pub start_date: NaiveDate,
}

impl Rampup {
    pub fn clamp_percentage(mut self) -> Self {
        let new_speed = match &self.speed {
            Speed::Percentage(p) => Speed::Percentage(p.clamp(-100.0, 100.0)),
            other @ Speed::ConstantSeconds(_) => other.clone(),
        };

        self.speed = new_speed;
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Speed {
    ConstantSeconds(i32),
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
            Ok(config) => config != Self::default(),
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

    pub(crate) fn reload(self) -> Self {
        let old_config = self;

        match Config::load(CONFIG_PATH) {
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

        let new_config = new_config.fix_values();
        new_config
            .check_correct()
            .wrap_err("New config has errors")?;

        Ok(new_config)
    }

    pub fn store(&self, path: &str) {
        match crate::store_as_toml(&self, path) {
            Ok(()) => (),
            Err(err) => eprintln!("Error while trying to store config: {err}"),
        };
    }

    fn fix_values(self) -> Self {
        Self(
            self.into_iter()
                .map(|(user, user_config)| {
                    (user.clone(), user_config.clone().clamp_rampup())
                })
                .collect(),
        )
    }

    fn check_correct(&self) -> Result<(), Error> {
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

    pub fn into_iter(self) -> impl Iterator<Item = (String, UserConfig)> {
        self.0.into_iter()
    }

    pub(crate) fn apply_rampup(self) -> Self {
        Self(
            self.into_iter()
                .map(|(user, mut user_config)| {
                    if let Some(rampup) = &user_config.rampup {
                        let today = Local::now().date_naive();
                        if today > rampup.start_date {
                            let n_days: i32 = (today - rampup.start_date)
                                .num_days()
                                .try_into()
                                .expect("n_days < 11Myears");
                            let old_time: u32 = user_config.allowed_seconds;

                            let new_time: u32 = match rampup.speed {
                                Speed::ConstantSeconds(s) => {
                                    old_time.saturating_add_signed(n_days * s)
                                }
                                Speed::Percentage(p) => {
                                    add_percentage(old_time, n_days, p)
                                }
                            };
                            user_config.allowed_seconds = new_time;
                        }
                    }
                    (user, user_config)
                })
                .collect(),
        )
    }

    pub(crate) fn store_rampedup(&self) {
        let rampedup: HashMap<String, u32> = self
            .0
            .iter()
            .map(|(user, user_config)| {
                (user.to_owned(), user_config.allowed_seconds)
            })
            .collect();

        match crate::store_as_toml(&rampedup, RAMPEDUP_PATH) {
            Ok(()) => (),
            Err(err) => {
                eprintln!("Error while trying to store rampedup: {err}");
            }
        };
    }
}

#[allow(clippy::cast_precision_loss)] // don't need > 23 bits precision
#[allow(clippy::cast_possible_truncation)] // new time < 8000 years
#[allow(clippy::cast_sign_loss)] // if percentage <= 100, res > 0
fn add_percentage(old_time: u32, n_days: i32, percentage: f32) -> u32 {
    let unrounded: f32 =
        old_time as f32 * (1.0 + percentage / 100.0).powi(n_days);

    unrounded.round() as u32
}
