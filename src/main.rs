use std::path::Path;

use chrono::Local;

use config::Config;

mod config;

const CONFIG_PATH: &str = "/home/focus/.config/time-guardian/config";

fn main() {
    let config = Config::initialize(Path::new(CONFIG_PATH));
    if config == Config::default() {
        println!("Default config found, please edit at {CONFIG_PATH}.");
    }
    dbg!(config);

    let now = Local::now();
    dbg!(now);
}
