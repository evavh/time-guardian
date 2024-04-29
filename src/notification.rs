use std::process::Command;

pub(crate) fn notify_user(target_name: &str, text: &str) {
    let users = get_logged_in_users();
    let (uid, name) = users
        .iter()
        .find(|(_uid, name)| name == target_name)
        .unwrap();

    notify(name, uid, text);
}

fn get_logged_in_users() -> Vec<(String, String)> {
    let users = Command::new("loginctl").output().unwrap().stdout;
    let users = String::from_utf8(users).unwrap();
    users
        .lines()
        .filter(|x| x.starts_with(' '))
        .map(|x| x.split(' ').filter(|x| !x.is_empty()))
        .map(|mut x| {
            (x.nth(1).unwrap().to_string(), x.next().unwrap().to_string())
        })
        .collect()
}

fn notify(username: &str, uid: &str, text: &str) {
    let command = format!("sudo -u {username} DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/{uid}/bus notify-send -t 5000 \"{text}\"");
    Command::new("sh").arg("-c").arg(command).output().unwrap();
}
