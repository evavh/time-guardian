use chrono::Local;
use std::{collections::HashMap, fs, thread, time::Duration};

mod user_management;

const CONFIG_PATH: &str = "/home/focus/.config/time-guardian/config";

fn main() {
    let config = fs::read_to_string(CONFIG_PATH).unwrap();

    let settings: HashMap<&str, usize> = config
        .lines()
        .map(|line| line.split(','))
        .map(|mut entry| {
            (
                entry.next().unwrap(),
                entry.next().unwrap().parse::<usize>().unwrap(),
            )
        })
        .inspect(|(user, _)| {
            assert!(
                user_management::exists(user),
                "Error in config: {user} doesn't exist"
            );
        })
        .collect();

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

            if user_management::is_active(user) {
                *spent_seconds.get_mut(user).unwrap() += 1;

                if spent_seconds[user] >= *allowed_seconds {
                    user_management::logout(user);
                }
            }
        }
    }
}

fn initialize_counting<'a>(
    settings: &'a HashMap<&'a str, usize>,
) -> HashMap<&'a str, usize> {
    settings.clone().into_keys().map(|user| (user, 0)).collect()
}
