use std::thread;
use std::time::Duration;

use crate::config::Config;
use crate::counter::Counter;
use crate::user_management::{is_active, logout};

mod config;
mod counter;
mod notification;
mod user_management;

fn main() {
    let mut config = Config::initialize_from_files();

    let mut counter = match Counter::load() {
        Ok(counter) => {
            if counter.is_outdated() {
                Counter::new(config.users())
            } else {
                counter
            }
        }
        Err(err) => {
            eprintln!("Error while loading counter: {err}, resetting");
            Counter::new(config.users())
        }
    };

    match counter.store() {
        Ok(()) => (),
        Err(err) => eprintln!("Error while trying to store counter: {err}"),
    };

    loop {
        if counter.is_outdated() {
            println!("New day, resetting");
            counter = Counter::new(config.users());

            config.reload();
        }

        thread::sleep(Duration::from_secs(1));

        for (user, config) in config.iter() {
            if is_active(user) {
                let count = counter
                    .spent_seconds
                    .get_mut(user)
                    .expect("Initialized from the hashmap, should be in there");
                *count += 1;

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
