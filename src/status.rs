use std::time::Duration;

use crate::{config::Config, counter::Counter, file_io};

pub(crate) fn spent(user: &str) {
    let spent = get_spent(user).as_secs_f64();

    println!("{spent}")
}

fn get_spent(user: &str) -> Duration {
    let counter = Counter::load().unwrap();
    let spent = if counter.is_outdated() {
        Duration::default()
    } else {
        counter.spent[user]
    };
    spent
}

pub(crate) fn status(user: &str) {
    let spent = get_spent(user);
    let allowed = Config::load(file_io::path::CONFIG).unwrap().allowed(user);

    println!("time left: {}", format(allowed - spent));
}

fn format(duration: Duration) -> String {
    let seconds = duration.as_secs();
    let hours = seconds / 3600;

    let minutes = seconds % 3600 / 60;
    let seconds = seconds % 3600 % 60;

    format!("{hours:02}:{minutes:02}:{seconds:02}")
}
