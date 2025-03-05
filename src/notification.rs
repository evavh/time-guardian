use std::process::Command;
use std::string::FromUtf8Error;

use color_eyre::{eyre::Context, Result};
use thiserror::Error;

use crate::logging::log_error;

#[derive(Error, Debug)]
enum Error {
    #[error("Couldn't parse loginctl output: {0}")]
    Parse(String),
    #[error("Io error")]
    Io(#[from] std::io::Error),
    #[error("Utf8 parsing error")]
    Utf8(#[from] FromUtf8Error),
    #[error("User {0} not found")]
    UserNotFound(String),
}

pub(crate) fn notify_user(target_name: &str, text: &str) {
    log_error(
        notify_user_err(target_name, text),
        &format!("Error while notifying user {target_name}"),
    );
}

fn notify_user_err(target_name: &str, text: &str) -> Result<()> {
    let users =
        get_logged_in_users().wrap_err("Couldn't get logged in users")?;

    let user = users.iter().find(|(_uid, name)| name == target_name);

    let (uid, name) = user
        .ok_or(Error::UserNotFound(target_name.to_owned()))
        .wrap_err("Couldn't find uid for user, not logged in?")?;

    notify(name, uid, text);

    Ok(())
}

// TODO: add Windows version
/// Returns Vec of (uid, username)
fn get_logged_in_users() -> Result<Vec<(String, String)>, Error> {
    let users = Command::new("loginctl").output()?.stdout;
    let users = String::from_utf8(users)?;
    let users: Result<Vec<(String, String)>, _> = users
        .lines()
        .filter(|x| x.starts_with(' '))
        .map(|x| x.split(' ').filter(|x| !x.is_empty()))
        .map(|mut x| {
            Ok((
                x.nth(1)
                    .ok_or(Error::Parse(x.clone().collect()))?
                    .to_string(),
                x.next().ok_or(Error::Parse(x.collect()))?.to_string(),
            ))
        })
        .collect();
    users
}

// TODO: use break-enforcer notify code for Linux
// TODO: add Windows version
fn notify(username: &str, uid: &str, text: &str) {
    let command = format!("sudo -u {username} DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/{uid}/bus notify-send -t 5000 \"{text}\"");

    log_error(
        Command::new("sh").arg("-c").arg(command).output().map(|_| ()),
        &format!("Error while notifying {username}"),
    );
}
