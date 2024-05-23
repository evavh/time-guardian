use std::thread;
use std::time::Duration;

use crate::config::Config;
use crate::user_management::{is_active, logout};

mod config;
mod counter;
mod notification;
mod user_management;

const CONFIG_PATH: &str = "/etc/time-guardian/config-dev.toml";
const PREV_CONFIG_PATH: &str = "/etc/time-guardian/prev-config-dev.toml";
const FALLBACK_CONFIG_PATH: &str =
    "/etc/time-guardian/fallback-config-dev.toml";
const TEMPLATE_CONFIG_PATH: &str =
    "/etc/time-guardian/template-config-dev.toml";
const STATUS_PATH: &str = "/var/lib/time-guardian/status-dev.toml";

fn main() {
    let mut config = initialize_config();

    let mut counter = match counter::Counter::load() {
        Ok(counter) => {
            if counter.is_outdated() {
                counter::Counter::new(config.users())
            } else {
                counter
            }
        }
        Err(err) => {
            eprintln!("Error while loading counter: {err}, resetting");
            counter::Counter::new(config.users())
        }
    };

    match counter.store() {
        Ok(()) => (),
        Err(err) => eprintln!("Error while trying to store counter: {err}"),
    };

    loop {
        // Reset on new day
        if counter.is_outdated() {
            println!("New day, resetting");
            counter = counter::Counter::new(config.users());

            let old_config = config.clone();
            config = match Config::load(CONFIG_PATH) {
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

        thread::sleep(Duration::from_secs(1));

        for (user, config) in config.iter() {
            if is_active(user) {
                *counter.spent_seconds.get_mut(user).expect("Initialized from the hashmap we iterate over, should be in there") += 1;
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

fn initialize_config() -> Config {
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
