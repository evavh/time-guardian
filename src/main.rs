use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use serde::Serialize;

use crate::config::{Config, UserConfig};
use crate::counter::Counter;
use crate::user_management::{is_active, logout};

mod config;
mod counter;
mod notification;
mod user_management;

const BREAK_IDLE_THRESHOLD: u64 = 10;

fn main() {
    let mut config = Config::initialize_from_files().apply_rampup();
    let mut counter = Counter::initialize(&config);
    config.store_rampedup();

    let mut break_enforcer = break_enforcer::Api::new();
    let mut retries = 0;

    loop {
        if counter.is_outdated() {
            eprintln!("New day, resetting");
            counter = Counter::new(config.users());

            config = config.reload().apply_rampup();
            config.store_rampedup();
        }

        thread::sleep(Duration::from_secs(1));

        for (user, user_config) in config.iter() {
            // Default to 0 idle = active
            let idle_time = get_idle_time(&mut break_enforcer, &mut retries);

            if is_active(user)
                && idle_time < Duration::from_secs(BREAK_IDLE_THRESHOLD)
            {
                counter = counter.increment(user);

                println!(
                    "{user} spent {} out of {}",
                    counter.spent_seconds[user], user_config.allowed_seconds
                );

                if counter.spent_seconds[user] >= user_config.allowed_seconds {
                    logout(user);
                    // This user doesn't need to be accounted for right now
                    continue;
                }

                counter.store();

                issue_warnings(&counter, user_config, user);
            }
        }
    }
}

fn get_idle_time(
    api_connection: &mut Result<break_enforcer::Api, break_enforcer::Error>,
    retries: &mut usize,
) -> Duration {
    match api_connection {
        Ok(ref mut break_enforcer) => match break_enforcer.idle_since() {
            Ok(time) => time,
            Err(err) => {
                if *retries < 3 {
                    eprintln!("Idle time reading failed: {err}");
                    *retries += 1
                }
                *api_connection = break_enforcer::Api::new();
                Duration::from_secs(0)
            }
        },
        Err(err) => {
            if *retries < 3 {
                eprintln!("Previous break enforcer connection failed: {err}");
                *retries += 1
            }
            *api_connection = break_enforcer::Api::new();
            Duration::from_secs(0)
        }
    }
}

fn issue_warnings(counter: &Counter, config: &UserConfig, user: &str) {
    // TODO: make short and long warnings different
    // (and multiple possible)

    let seconds_left = config
        .allowed_seconds
        .saturating_sub(counter.spent_seconds[user]);

    if seconds_left == config.short_warning_seconds
        || seconds_left == config.long_warning_seconds
    {
        notification::notify_user(
            user,
            &format!("You will be logged out in {seconds_left} seconds!",),
        );
    }
}

fn store_as_toml(
    object: &impl Serialize,
    path: &str,
) -> Result<(), std::io::Error> {
    let toml = toml::to_string(&object)
        .expect("Serializing failed, probably an error in toml");

    if !PathBuf::from(path)
        .parent()
        .expect("This path should have a parent")
        .exists()
    {
        std::fs::create_dir_all(
            PathBuf::from(path)
                .parent()
                .expect("This path should have a parent"),
        )?;
    }
    std::fs::write(path, toml)?;
    Ok(())
}
