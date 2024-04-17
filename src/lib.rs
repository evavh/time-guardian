use std::collections::HashMap;
use std::thread;
use std::time::Duration;

use chrono::Local;
use serde_derive::{Deserialize, Serialize};

use crate::user_management::{exists, is_active, logout};

use self::user_management::list_users;

mod user_management;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub total_per_day: HashMap<String, usize>,
}

impl Default for Config {
    fn default() -> Self {
        let total_per_day: HashMap<String, usize> =
            list_users().into_iter().map(|user| (user, 86400)).collect();
        Self { total_per_day }
    }
}

pub fn run(config: Config) -> ! {
    let total_per_day = config.total_per_day;

    let mut spent_seconds = initialize_counting(&total_per_day);
    let mut accounted_date = Local::now().date_naive();

    loop {
        let current_date = Local::now().date_naive();
        // Reset on new day
        if current_date != accounted_date {
            println!("New day, resetting");
            spent_seconds = initialize_counting(&total_per_day);
            accounted_date = current_date;
        }

        thread::sleep(Duration::from_secs(1));

        for (user, allowed_seconds) in &total_per_day {
            println!(
                "User {user} has now spent {}/{}s",
                spent_seconds[user], allowed_seconds
            );

            if is_active(user) {
                *spent_seconds.get_mut(user).unwrap() += 1;

                if spent_seconds[user] >= *allowed_seconds {
                    logout(user);
                }
            }
        }
    }
}

pub fn check_correct(config: &Config) {
    let Config { total_per_day } = config;

    for user in total_per_day.keys() {
        assert!(exists(user), "Error in config: user {user} does not exist");
    }
}

pub(crate) fn initialize_counting(
    settings: &HashMap<String, usize>,
) -> HashMap<String, usize> {
    settings.clone().into_keys().map(|user| (user, 0)).collect()
}
