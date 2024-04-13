use std::{collections::HashMap, path::Path};

use chrono::Local;

const CONFIG_PATH: &str = "/home/focus/.config/time-guardian/config";

fn main() {
    let config = std::fs::read_to_string(CONFIG_PATH).unwrap();

    let entries: HashMap<&str, usize> = config
        .lines()
        .map(|line| line.split(","))
        .map(|mut entry| {
            (
                entry.next().unwrap(),
                entry.next().unwrap().parse::<usize>().unwrap(),
            )
        })
        .collect();

    dbg!(entries);

    let now = Local::now();
    dbg!(now);
}
