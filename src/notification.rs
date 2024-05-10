use std::process::Command;
use std::string::FromUtf8Error;
use std::sync::Arc;

use color_eyre::{eyre::Context, Result};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
enum Error {
    #[error("Couldn't parse loginctl output: {0}")]
    Parse(String),
    #[error("Io error")]
    Io(#[from] Arc<std::io::Error>),
    #[error("Utf8 parsing error")]
    Utf8(#[from] FromUtf8Error),
}

pub(crate) fn notify_user(target_name: &str, text: &str) {
    match notify_user_err(target_name, text) {
        Ok(()) => (),
        Err(err) => eprintln!("{err}"),
    }
}

fn notify_user_err(target_name: &str, text: &str) -> Result<()> {
    let users = get_logged_in_users().wrap_err("Couldn't get logged in users")?;

    let user = users
        .iter()
        .cloned()
        .find(|(_uid, name)| name == target_name);

    match user {
        Some((uid, name)) => notify(&name, &uid, text),
        None => {
            eprintln!("Couldn't find uid for {target_name}, not logged in?")
        }
    };
    Ok(())
}

fn get_logged_in_users() -> Result<Vec<(String, String)>, Error> {
    let users = Command::new("loginctl").output().map_err(Arc::new)?.stdout;
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
                x.next()
                    .ok_or(Error::Parse(x.collect()))?
                    .to_string(),
            ))
        })
        .collect();
    users
}

fn notify(username: &str, uid: &str, text: &str) {
    let command = format!("sudo -u {username} DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/{uid}/bus notify-send -t 5000 \"{text}\"");

    match Command::new("sh").arg("-c").arg(command).output() {
        Ok(_) => (),
        Err(e) => eprintln!("Error while notifying {username}: {e}"),
    }
}
