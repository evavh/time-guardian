use crate::file_io;
use crate::user;

use chrono::{Local, NaiveDate};
use color_eyre::{eyre::Context, Result};
use serde_derive::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSecondsWithFrac};

use std::{collections::HashMap, time::Duration};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("User {0} doesn't exist")]
    UserDoesntExist(String),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Config(HashMap<String, UserConfig>);

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[allow(clippy::module_name_repetitions)]
pub struct UserConfig {
    // TODO: make warnings a user-editable setting
    #[serde_as(as = "DurationSecondsWithFrac<f64>")]
    pub short_warning: Duration,
    #[serde_as(as = "DurationSecondsWithFrac<f64>")]
    pub long_warning: Duration,
    #[serde_as(as = "DurationSecondsWithFrac<f64>")]
    pub allowed: Duration,
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
        let users = match user::list_users() {
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
            short_warning: Duration::from_secs(30),
            long_warning: Duration::from_secs(300),
            allowed: Duration::from_secs(86400),
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
        if match Config::load(file_io::path::TEMPLATE_CONFIG) {
            Ok(config) => config != Self::default(),
            Err(_) => true,
        } {
            println!("Writing new template config");
            Config::default().store(file_io::path::TEMPLATE_CONFIG);
        }

        match Config::load(file_io::path::CONFIG) {
            Ok(config) => config,
            Err(err) => {
                eprintln!(
                    "Error while initially loading config, using previous config\nCause: {err:?}"
                );
                match Config::load(file_io::path::PREV_CONFIG) {
                    Ok(config) => config,
                    Err(err) => {
                        eprintln!("Error while loading previous config on startup, using fallback\nCause: {err:?}");
                        Config::load(file_io::path::FALLBACK_CONFIG).unwrap()
                    }
                }
            }
        }
    }

    pub(crate) fn reload(self) -> Self {
        let old_config = self;

        match Config::load(file_io::path::CONFIG) {
            Ok(new_config) => {
                Config::store(&new_config, file_io::path::PREV_CONFIG);
                new_config
            }
            Err(err) => {
                eprintln!("Error loading config: {err:?}");
                old_config
            }
        }
    }

    pub fn load(path: &str) -> Result<Self> {
        let new_config: Self = file_io::load(path)?;

        let new_config = new_config.fix_values();
        new_config
            .check_correct()
            .wrap_err("New config has errors")?;

        Ok(new_config)
    }

    pub fn store(&self, path: &str) {
        match file_io::store(&self, path) {
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
            if !user::exists(&user) {
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

    pub fn allowed(&self, user: &str) -> Duration {
        self.0[user].allowed
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
                            let old_seconds: u32 = user_config
                                .allowed
                                .as_secs()
                                .try_into()
                                .expect("allowed time < 22Myears");

                            let new_seconds: u32 = match rampup.speed {
                                Speed::ConstantSeconds(s) => old_seconds
                                    .saturating_add_signed(n_days * s),
                                Speed::Percentage(p) => {
                                    add_percentage(old_seconds, n_days, p)
                                }
                            };
                            user_config.allowed =
                                Duration::from_secs(new_seconds.into());
                        }
                    }
                    (user, user_config)
                })
                .collect(),
        )
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
