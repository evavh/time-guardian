use chrono::Local;
use std::{collections::HashMap, fs, process::Command, thread, time::Duration};

const CONFIG_PATH: &str = "/home/focus/.config/time-guardian/config";

fn main() {
    let config = fs::read_to_string(CONFIG_PATH).unwrap();

    let settings: HashMap<&str, usize> = config
        .lines()
        .map(|line| line.split(","))
        .map(|mut entry| {
            (
                entry.next().unwrap(),
                entry.next().unwrap().parse::<usize>().unwrap(),
            )
        })
        .inspect(|(user, _)| {
            assert!(exists(user), "Error in config: {user} doesn't exist")
        })
        .collect();

    let mut spent_seconds = initialize_counting(&settings);
    let mut accounted_date = Local::now().date_naive();

    loop {
        let current_date = Local::now().date_naive();
        // Reset on new day
        if current_date != accounted_date {
            println!("New day, resetting");
            spent_seconds = initialize_counting(&settings);
            accounted_date = current_date;
        }

        thread::sleep(Duration::from_secs(1));

        for (user, allowed_seconds) in &settings {
            println!("User {user} has now spent {}s", spent_seconds[user]);

            if is_active(user) {
                *spent_seconds.get_mut(user).unwrap() += 1;

                if spent_seconds[user] >= *allowed_seconds {
                    logout(user);
                }
            }
        }
    }
}

fn initialize_counting<'a>(
    settings: &'a HashMap<&'a str, usize>,
) -> HashMap<&'a str, usize> {
    settings
        .clone()
        .into_iter()
        .map(|(user, _t)| (user, 0))
        .collect()
}

fn logout(user: &str) {
    println!("Logging out user {user}");
    // Command::new("loginctl")
    //     .arg("terminate-user")
    //     .arg(user)
    //     .output()
    //     .unwrap();
}

fn exists(user: &str) -> bool {
    fs::read_to_string("/etc/passwd")
        .unwrap()
        .contains(&format!("{user}:"))
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
