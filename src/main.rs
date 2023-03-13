use chrono::Local;

mod config;

fn main() {
    let config = config::initialize();
    dbg!(config);

    let now = Local::now();
    dbg!(now);
}
