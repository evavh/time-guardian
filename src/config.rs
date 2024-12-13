use std::{collections::HashMap, time::Duration};

use color_eyre::{eyre::Context, Result};
use jiff::civil::Date;
use jiff::Zoned;
use log::{error, info};
use serde_derive::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSecondsWithFrac};
use strum::{Display, VariantArray};

use crate::file_io;
use crate::logging::log_error;
use crate::time_slot::TimeSlot;
use crate::user;

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
    pub rampup: Option<Rampup>,
    // TODO: change to jiff Weekday when it supports serde...
    days: HashMap<Vec<Weekday>, DayConfig>,
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Display,
    VariantArray,
)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl From<jiff::civil::Weekday> for Weekday {
    fn from(value: jiff::civil::Weekday) -> Self {
        match value {
            jiff::civil::Weekday::Monday => Weekday::Monday,
            jiff::civil::Weekday::Tuesday => Weekday::Monday,
            jiff::civil::Weekday::Wednesday => Weekday::Monday,
            jiff::civil::Weekday::Thursday => Weekday::Monday,
            jiff::civil::Weekday::Friday => Weekday::Monday,
            jiff::civil::Weekday::Saturday => Weekday::Monday,
            jiff::civil::Weekday::Sunday => Weekday::Monday,
        }
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct DayConfig {
    #[serde_as(as = "DurationSecondsWithFrac<f64>")]
    total_allowed: Duration,
    time_slots: Option<Vec<TimeSlot>>,
}

impl Default for DayConfig {
    fn default() -> Self {
        Self {
            total_allowed: Duration::from_secs(86400),
            time_slots: Some(vec![TimeSlot::default()]),
        }
    }
}

impl UserConfig {
    pub fn clamp_rampup(mut self) -> Self {
        self.rampup = self.rampup.map(Rampup::clamp_percentage);
        self
    }

    pub fn total_allowed_today(&self) -> Duration {
        self.todays_config().total_allowed
    }

    pub fn timeslots_today(&self) -> Option<Vec<TimeSlot>> {
        self.todays_config().time_slots
    }

    fn todays_config(&self) -> DayConfig {
        let current_weekday: Weekday = Zoned::now().weekday().into();
        let day_configs: Vec<DayConfig> = self
            .days
            .clone()
            .into_iter()
            .filter(|(days, _)| days.contains(&current_weekday))
            .map(|(_, config)| config)
            .collect();
        if day_configs.len() > 1 {
            println!(
                "{current_weekday} is in config multiple times,
                using first occurence"
            );
        }
        if day_configs.len() == 0 {
            println!(
                "{current_weekday} is not in config!
                Using default (no blocking)"
            );
            return DayConfig::default();
        }
        day_configs.into_iter().next().expect("Checked for empty")
    }

    fn timeslots_right_now(&self) -> Option<Vec<TimeSlot>> {
        self.todays_config().time_slots.as_ref().map(|x| {
            x.iter()
                .filter(|&slot| slot.contains(Zoned::now()))
                .cloned()
                .collect()
        })
    }

    pub fn now_within_timeslot(&self) -> bool {
        let current_timeslots = self.timeslots_right_now();

        current_timeslots.is_none()
            || current_timeslots.is_some_and(|v| !v.is_empty())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Rampup {
    pub speed: Speed,
    pub start_date: Date,
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
                error!("Couldn't list users in home: {err:?}");
                vec!["example_user".to_owned()]
            }
        };

        let rampup = Rampup {
            speed: Speed::ConstantSeconds(1),
            start_date: Date::new(2024, 5, 1).expect("Date exists"),
        };
        let days =
            HashMap::from([(Weekday::VARIANTS.to_vec(), DayConfig::default())]);
        let user_config = UserConfig {
            short_warning: Duration::from_secs(30),
            long_warning: Duration::from_secs(300),
            rampup: Some(rampup),
            days,
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
            info!("Writing new template config");
            Config::default().store(file_io::path::TEMPLATE_CONFIG);
        }

        match Config::load(dbg!(file_io::path::CONFIG)) {
            Ok(config) => config,
            Err(err) => {
                error!(
                    "Error while initially loading config, using previous config\nCause: {err:?}"
                );
                match Config::load(file_io::path::PREV_CONFIG) {
                    Ok(config) => config,
                    Err(err) => {
                        error!("Error while loading previous config on startup, using fallback\nCause: {err:?}");
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
                error!("Error loading config: {err:?}");
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
        log_error(
            file_io::store(&self, path),
            "Error while trying to store config",
        );
    }

    fn fix_values(self) -> Self {
        Self(
            self.into_iter()
                // Remove root user, should never be guarded
                .filter(|(user, _)| user != "root")
                // Clamp rampup
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

    pub fn user(&self, user: &str) -> &UserConfig {
        &self.0[user]
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &UserConfig)> + '_ {
        self.0.iter()
    }

    pub fn into_iter(self) -> impl Iterator<Item = (String, UserConfig)> {
        self.0.into_iter()
    }

    pub fn allowed(&self, user: &str) -> Duration {
        self.0[user].todays_config().total_allowed
    }

    pub(crate) fn apply_rampup(self) -> Self {
        Self(
            self.into_iter()
                .map(|(user, user_config)| {
                    let mut day_config = user_config.todays_config();
                    if let Some(rampup) = &user_config.rampup {
                        let today = Zoned::now().datetime().date();
                        if today > rampup.start_date {
                            let n_days: i32 =
                                (today - rampup.start_date).get_days();
                            let old_seconds: u32 = day_config
                                .total_allowed
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
                            day_config.total_allowed =
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
