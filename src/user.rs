use std::ffi::OsString;
use std::process::Command;
use std::time::Duration;
use std::{fs, thread};

use color_eyre::Result;
use thiserror::Error;

const MAX_RETRIES: usize = 5;

#[derive(Error, Debug)]
enum Error {
    #[error("Unexpected error from loginctl")]
    Command(#[from] std::io::Error),
    #[error("Utf8 parsing error")]
    Utf8(#[from] core::str::Utf8Error),
    #[error("Unexpected error from loginctl")]
    Loginctl(String),
    #[error("OsString couldn't be converted")]
    OsString(OsString),
}

pub(crate) fn list_users() -> Result<Vec<String>> {
    let users: Result<Vec<String>> = fs::read_dir("/home")?
        .map(|d| Ok(d?.file_name()))
        .map(|s: Result<OsString>| {
            Ok(s?.into_string().map_err(Error::OsString)?)
        })
        .collect();
    users
}

pub(crate) fn logout(user: &str) {
    println!("Logging out user {user}");
    let mut retries = 0;

    while retries < MAX_RETRIES {
        let output = Command::new("loginctl")
            .arg("terminate-user")
            .arg(user)
            .output();

        match output {
            Ok(_) => return,
            Err(err) => eprintln!("Error while trying to logout {user}: {err}"),
        }

        retries += 1;
        thread::sleep(Duration::from_secs(5));
    }

    eprintln!("Reached maximum retries for logout");
}

pub(crate) fn exists(user: &str) -> bool {
    match fs::read_to_string("/etc/passwd") {
        Ok(passwd) => passwd
            .lines()
            .any(|line| line.starts_with(&format!("{user}:"))),
        // Default to user exists
        Err(err) => {
            eprintln!("Couldn't read /etc/passwd: {err}");
            true
        }
    }
}

pub(crate) fn is_active(user: &str) -> bool {
    match is_active_err(user) {
        Ok(res) => res,
        // Default to active
        Err(err) => {
            eprintln!("Active checking encountered an error {err}, defaulting to active");
            true
        }
    }
}

fn is_active_err(user: &str) -> Result<bool, Error> {
    let output = Command::new("loginctl")
        .arg("show-user")
        .arg(user)
        .arg("--property=State")
        .output()?;

    let err = std::str::from_utf8(&output.stderr)?;
    if !err.is_empty() && !err.contains("is not logged in or lingering") {
        Error::Loginctl(format!(
            "Unknown loginctl error, user: {user}, output: {output:?}"
        ));
    }
    let state = std::str::from_utf8(&output.stdout)?;

    Ok(state.contains("active"))
}