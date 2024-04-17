use chrono::Local;
use serde_derive::{Deserialize, Serialize};
use std::{collections::HashMap, thread, time::Duration};

use crate::user_management::{exists, is_active, list_users, logout};

mod user_management;

#[derive(Serialize, Deserialize)]
struct Config {
    total_per_day: HashMap<String, usize>,
}

impl Default for Config {
    fn default() -> Self {
        let total_per_day: HashMap<String, usize> =
            list_users().into_iter().map(|user| (user, 86400)).collect();
        Self { total_per_day }
    }
}

fn main() {
    println!(
        "Using config file: {:?}",
        confy::get_configuration_file_path("time-guardian", None).unwrap()
    );
    let config = confy::load("time-guardian", Some("config")).unwrap();
    check_correct(&config);
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

fn check_correct(config: &Config) {
    let Config { total_per_day } = config;

    for (user, _) in total_per_day {
        assert!(exists(&user), "Error in config: user {user} does not exist");
    }
}

fn initialize_counting(
    settings: &HashMap<String, usize>,
) -> HashMap<String, usize> {
    settings.clone().into_keys().map(|user| (user, 0)).collect()
}
