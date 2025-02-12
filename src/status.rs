use std::time::Duration;

use crate::{config::Config, file_io, tracker::Tracker};

pub(crate) fn spent(user: &str) {
    let spent = get_spent(user).as_secs_f64();

    println!("{spent}");
}

fn get_spent(user: &str) -> Duration {
    let tracker = Tracker::load().unwrap();

    if tracker.is_outdated() {
        Duration::default()
    } else {
        match tracker.counter.get(user) {
            Some(user) => user,
            None => {
                eprintln!("Couldn't get {user} from {:?}", tracker.counter);
                return Duration::MAX;
            }
        }
        .total_spent
    }
}

pub(crate) fn status(user: &str) {
    let spent = get_spent(user);
    let rampedup = Config::load(file_io::path::RAMPEDUP).unwrap();
    let allowed = rampedup.allowed(user);

    println!("time left: {}", format(allowed.saturating_sub(spent)));
}

fn format(duration: Duration) -> String {
    let seconds = duration.as_secs();
    let hours = seconds / 3600;

    let minutes = seconds % 3600 / 60;
    let seconds = seconds % 3600 % 60;

    format!("{hours:02}:{minutes:02}:{seconds:02}")
}
