use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use chrono::Local;
use chrono::NaiveDate;
use color_eyre::Result;
use serde_derive::{Deserialize, Serialize};

use crate::STATUS_PATH;

#[derive(Serialize, Deserialize)]
pub(crate) struct Counter {
    pub(crate) date: NaiveDate,
    pub(crate) spent_seconds: HashMap<String, usize>,
}

impl Counter {
    pub(crate) fn new(users: impl Iterator<Item=String>) -> Self {
        let spent_seconds =
            users.map(|user| (user, 0)).collect();

        Self {
            date: Local::now().date_naive(),
            spent_seconds,
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

    pub(crate) fn store(&self) -> Result<(), std::io::Error> {
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
