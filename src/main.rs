use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use serde::Serialize;

use crate::config::Config;
use crate::counter::Counter;
use crate::user_management::{is_active, logout};

use self::config::UserConfig;

mod config;
mod counter;
mod notification;
mod user_management;

fn main() {
    let mut config = Config::initialize_from_files();
    let mut counter = Counter::initialize(&config);
    config.apply_rampup();
    config.store_rampedup();

    loop {
        if counter.is_outdated() {
            eprintln!("New day, resetting");
            counter = Counter::new(config.users());

            config.reload();
            config.apply_rampup();
            config.store_rampedup();
        }

        thread::sleep(Duration::from_secs(1));

        for (user, user_config) in config.iter() {
            if is_active(user) {
                counter.increment(user);

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
