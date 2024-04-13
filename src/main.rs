use std::{collections::HashMap, process::Command};

const CONFIG_PATH: &str = "/home/focus/.config/time-guardian/config";

fn main() {
    let config = std::fs::read_to_string(CONFIG_PATH).unwrap();

    let settings: HashMap<&str, usize> = config
        .lines()
        .map(|line| line.split(","))
        .map(|mut entry| {
            (
                entry.next().unwrap(),
                entry.next().unwrap().parse::<usize>().unwrap(),
            )
        })
        .collect();

    dbg!(is_active("focus"));
}

fn is_active(user: &str) -> bool {
    let output = Command::new("loginctl")
        .arg("show-user")
        .arg(user)
        .arg("--property=State")
        .output()
        .unwrap();

    let err = std::str::from_utf8(&output.stderr).unwrap();
    if !err.is_empty() {
        assert!(
            err.contains("is not logged in or lingering"),
            "Unknown loginctl error, output: {output:?}"
        );
    }
    let state = std::str::from_utf8(&output.stdout).unwrap();

    state.contains("active")
}
