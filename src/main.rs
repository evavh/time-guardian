use std::{collections::HashMap, process::Command, thread, time::Duration};

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

    let mut spent_seconds: HashMap<&str, usize> = settings
        .clone()
        .into_iter()
        .map(|(user, _t)| (user, 0))
        .collect();

    loop {
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

fn logout(user: &str) {
    println!("Logging out user {user}");
    Command::new("loginctl")
        .arg("terminate-user")
        .arg(user)
        .output()
        .unwrap();
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
