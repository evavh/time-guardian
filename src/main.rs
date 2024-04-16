use chrono::Local;
use std::{collections::HashMap, fs, thread, time::Duration};

use crate::user_management::{exists, is_active, logout};

mod user_management;

const CONFIG_PATH: &str = "/home/focus/.config/time-guardian/config";

fn main() {

    let settings = load_config();

    let mut spent_seconds = initialize_counting(&settings);
    let mut accounted_date = Local::now().date_naive();

    loop {
        let current_date = Local::now().date_naive();
        // Reset on new day
        if current_date != accounted_date {
            println!("New day, resetting");
            spent_seconds = initialize_counting(&settings);
            accounted_date = current_date;
        }

        thread::sleep(Duration::from_secs(1));

        for (user, allowed_seconds) in &settings {
            println!("User {user} has now spent {}s", spent_seconds[user]);

            if is_active(user) {
                *spent_seconds.get_mut(user).unwrap() += 1;

                if spent_seconds[user] >= *allowed_seconds {
                    logout(user);
                }
            }
        }
    }
}

fn load_config() -> HashMap<String, usize> {
    let config = fs::read_to_string(CONFIG_PATH).unwrap();

    config
        .lines()
        .map(|line| line.split(','))
        .map(|mut entry| {
            (
                entry.next().unwrap().to_owned(),
                entry.next().unwrap().parse::<usize>().unwrap(),
            )
        })
        .inspect(|(user, _)| {
            assert!(
                exists(user),
                "Error in config: {user} doesn't exist"
            );
        })
        .collect()
}

fn initialize_counting(
    settings: &HashMap<String, usize>,
) -> HashMap<String, usize> {
    settings.clone().into_keys().map(|user| (user, 0)).collect()
}
