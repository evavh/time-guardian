use std::thread;
use std::time::Duration;
use std::time::Instant;

use log::{error, info, trace};

use crate::config::Config;
use crate::config::UserConfig;
use crate::counter::Counter;
use crate::file_io::path;
use crate::notification;
use crate::user;
use crate::BREAK_IDLE_THRESHOLD;

pub(crate) fn run() {
    let mut config = Config::initialize_from_files().apply_rampup();
    let mut counter = Counter::initialize(&config);
    config.store(path::RAMPEDUP);

    let mut break_enforcer = break_enforcer::Api::new();
    let mut retries = 0;

    let mut now = Instant::now();

    loop {
        if counter.is_outdated() {
            info!("New day, resetting");
            counter = Counter::new(config.users());

            config = config.reload().apply_rampup();
            config.store(path::RAMPEDUP);
        }

        thread::sleep(Duration::from_secs(1));
        let elapsed = now.elapsed();
        now = Instant::now();

        for (user, user_config) in config.iter() {
            // Default to 0 idle = active
            let idle_time = get_idle_time(&mut break_enforcer, &mut retries);

            if user::is_active(user)
                && idle_time < Duration::from_secs(BREAK_IDLE_THRESHOLD)
            {
                counter = counter.add(user, elapsed);

                trace!(
                    "{user} spent {:.1?} out of {:?}",
                    counter.spent[user],
                    user_config.allowed
                );

                if counter.spent[user] >= user_config.allowed {
                    user::logout(user);
                    // This user doesn't need to be accounted for right now
                    continue;
                }

                counter.store();

                issue_warnings(&counter, user_config, user);
            }
        }
    }
}

pub(crate) fn get_idle_time(
    api_connection: &mut Result<break_enforcer::Api, break_enforcer::Error>,
    retries: &mut usize,
) -> Duration {
    match api_connection {
        Ok(ref mut break_enforcer) => match break_enforcer.idle_since() {
            Ok(time) => time,
            Err(err) => {
                if *retries < 3 {
                    error!("Idle time reading failed: {err}");
                    *retries += 1;
                }
                *api_connection = break_enforcer::Api::new();
                Duration::default()
            }
        },
        Err(err) => {
            if *retries < 3 {
                error!("Previous break enforcer connection failed: {err}");
                *retries += 1;
            }
            *api_connection = break_enforcer::Api::new();
            Duration::default()
        }
    }
}

pub(crate) fn issue_warnings(
    counter: &Counter,
    config: &UserConfig,
    user: &str,
) {
    // TODO: make short and long warnings different
    // (and multiple possible)

    let time_left = config.allowed.saturating_sub(counter.spent[user]);

    if time_left == config.short_warning || time_left == config.long_warning {
        notification::notify_user(
            user,
            &format!("You will be logged out in {time_left:.0?} seconds!",),
        );
    }
}
