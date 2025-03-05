use std::thread;
use std::time::Duration;
use std::time::Instant;

#[allow(unused_imports)]
use log::{error, info, trace};

use crate::config::Config;
use crate::config::UserConfig;
use crate::file_io::path;
use crate::notification;
use crate::tracker::Tracker;
use crate::user;
use crate::BREAK_IDLE_THRESHOLD;

pub(crate) fn run() {
    let mut config = Config::initialize_from_files().apply_rampup();
    let mut tracker = Tracker::initialize(&config);
    config.store(path::RAMPEDUP);

    #[cfg(target_os = "linux")]
    let mut break_enforcer = break_enforcer::Api::new();
    #[cfg(target_os = "linux")]
    let mut retries = 0;

    let mut now = Instant::now();

    loop {
        if tracker.is_outdated() {
            info!("New day, resetting");
            tracker = Tracker::new(&config);

            config = config.reload().apply_rampup();
            config.store(path::RAMPEDUP);
        }

        thread::sleep(Duration::from_secs(1));
        let elapsed = now.elapsed();
        now = Instant::now();

        for (user, user_config) in config.iter() {
            // Default to 0 idle = active
            #[cfg(target_os = "linux")]
            let idle_time = get_idle_time(&mut break_enforcer, &mut retries);
            #[cfg(target_os = "windows")]
            let idle_time = Duration::default();

            if user::is_active(user)
                && idle_time < Duration::from_secs(BREAK_IDLE_THRESHOLD)
            {
                // TODO? limitation: only reloads new timeslot settings on new day
                tracker.add(user, elapsed);

                trace!(
                    "{user} spent {:.1?} out of {:?}",
                    tracker.counter[user].total_spent,
                    user_config.total_allowed_today()
                );
                trace!("Timeslots: {:#?}", tracker.counter[user].time_slots);

                if tracker.counter[user].total_spent
                    >= user_config.total_allowed_today()
                    || tracker.timeslot_over_time(&config, user)
                    || !user_config.now_within_timeslot()
                {
                    user::logout(user);
                    // This user doesn't need to be accounted for right now
                    continue;
                }

                tracker.store();

                issue_warnings(&tracker, user_config, user);
            }
        }
    }
}

#[cfg(target_os = "linux")]
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
    tracker: &Tracker,
    config: &UserConfig,
    user: &str,
) {
    // TODO: make short and long warnings different
    // (and multiple possible)

    let time_left = config
        .total_allowed_today()
        .saturating_sub(tracker.counter[user].total_spent);

    if time_left == config.short_warning || time_left == config.long_warning {
        notification::notify_user(
            user,
            &format!("You will be logged out in {time_left:.0?} seconds!",),
        );
    }
}
