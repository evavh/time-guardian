use std::path::Path;

use chrono::Local;

const CONFIG_PATH: &str = "/home/focus/.config/time-guardian/config";

fn main() {
    let now = Local::now();
    dbg!(now);
}
