use std::process::Command;

use color_eyre::{eyre::Context, Result};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Couldn't parse loginctl output: {0}")]
    ParseLoginctlOutput(String),
}

pub(crate) fn notify_user(target_name: &str, text: &str) {
    match notify_user_err(target_name, text) {
        Ok(()) => (),
        Err(err) => eprintln!("{err}"),
    }
}

fn notify_user_err(target_name: &str, text: &str) -> Result<()> {
    let users =
        get_logged_in_users().wrap_err("Couldn't get logged in users")?;

    let user = users
        .iter()
        .cloned()
        .filter(|x| x.is_ok())
        .map(|x| x.expect("Filtered on ok"))
        .find(|(_uid, name)| name == target_name);

    let first_error = users.iter().cloned().filter(|x| x.is_err()).next();
    match first_error {
        Some(Err(err)) => return Err(err).wrap_err("Error while parsing loginctl"),
        None => (),
        Some(Ok(_)) => unreachable!(),

    };

    match user {
        Some((uid, name)) => notify(&name, &uid, text),
        None => {
            eprintln!("Couldn't find uid for {target_name}, not logged in?")
        }
    };
    Ok(())
}

fn get_logged_in_users() -> Result<Vec<Result<(String, String), Error>>> {
    let users = Command::new("loginctl").output()?.stdout;
    let users = String::from_utf8(users)?;
    Ok(users
        .lines()
        .filter(|x| x.starts_with(' '))
        .map(|x| x.split(' ').filter(|x| !x.is_empty()))
        .map(|mut x| {
            Ok((
                x.nth(1)
                    .ok_or(Error::ParseLoginctlOutput(x.clone().collect()))?
                    .to_string(),
                x.next()
                    .ok_or(Error::ParseLoginctlOutput(x.collect()))?
                    .to_string(),
            ))
        })
        .collect())
}

fn notify(username: &str, uid: &str, text: &str) {
    let command = format!("sudo -u {username} DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/{uid}/bus notify-send -t 5000 \"{text}\"");

    match Command::new("sh").arg("-c").arg(command).output() {
        Ok(_) => (),
        Err(e) => eprintln!("Error while notifying {username}: {e}"),
    }
}
