use std::ffi::OsString;
#[cfg(target_os = "linux")]
use std::process::Command;
use std::string::FromUtf16Error;
#[allow(unused_imports)]
use std::time::Duration;
#[allow(unused_imports)]
use std::{fs, thread};

use color_eyre::Result;
#[allow(unused_imports)]
use log::{error, info, warn};
use thiserror::Error;

#[cfg(target_os = "windows")]
use windows::core::PWSTR;
#[cfg(target_os = "windows")]
#[cfg(feature = "deploy")]
use windows::Win32::System::RemoteDesktop::WTSLogoffSession;
#[cfg(target_os = "windows")]
#[cfg(feature = "deploy")]
use windows::Win32::System::RemoteDesktop::WTS_CURRENT_SERVER_HANDLE;
#[cfg(target_os = "windows")]
use windows::Win32::System::WindowsProgramming;

#[cfg(target_os = "windows")]
use crate::session;

#[allow(dead_code)]
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

    #[error("Error getting username from Windows")]
    GetUserName(String),
    #[error("Error converting string to Utf16")]
    Utf16(#[from] FromUtf16Error),
    #[cfg(target_os = "windows")]
    #[error("Error from Windows")]
    Windows(#[from] windows_core::Error),
}

// TODO: add Windows version
pub(crate) fn list_users() -> Result<Vec<String>> {
    #[cfg(target_os = "linux")]
    let home_dir = "/home";
    #[cfg(target_os = "windows")]
    let home_dir = "C:\\Users";
    let users = fs::read_dir(home_dir)?.map(|d| Ok(d?.file_name())).map(
        |s: Result<OsString>| Ok(s?.into_string().map_err(Error::OsString)?),
    );

    #[cfg(target_os = "windows")]
    let users =
        users.filter(|u| u.as_ref().unwrap_or(&String::new()) != "Public");

    users.collect()
}

#[cfg(feature = "deploy")]
#[cfg(target_os = "linux")]
pub(crate) fn logout(user: &str) {
    info!("Logging out user {user}");
    let mut retries = 0;

    while retries < MAX_RETRIES {
        let output = Command::new("loginctl")
            .arg("terminate-user")
            .arg(user)
            .output();

        match output {
            Ok(_) => return,
            Err(err) => error!("Error while trying to logout {user}: {err}"),
        }

        retries += 1;
        thread::sleep(Duration::from_secs(5));
    }

    warn!("Reached maximum retries for logout");
}

#[cfg(feature = "deploy")]
#[cfg(target_os = "windows")]
/// Unsafe
pub(crate) fn logout(user: &str) {
    let active_consoles =
        get_active_consoles().filter(|s| s.username == Some(user.to_string()));

    for session in active_consoles {
        println!("Logging out {session:?}");
        unsafe {
            WTSLogoffSession(WTS_CURRENT_SERVER_HANDLE, session.id, false)
                .unwrap();
        }
    }
}

#[cfg(target_os = "windows")]
fn get_active_consoles() -> impl Iterator<Item = session::Session> {
    session::get_sessions()
        .into_iter()
        .filter(|s| s.active)
        .filter(|s| s.station_name == "Console")
}

#[cfg(not(feature = "deploy"))]
pub(crate) fn logout(user: &str) {
    println!("Would log out user {user}, not deployed");
}

#[cfg(target_os = "linux")]
pub(crate) fn exists(user: &str) -> bool {
    match fs::read_to_string("/etc/passwd") {
        Ok(passwd) => passwd
            .lines()
            .any(|line| line.starts_with(&format!("{user}:"))),
        // Default to user exists
        Err(err) => {
            error!("Couldn't read /etc/passwd: {err}");
            true
        }
    }
}

#[cfg(target_os = "windows")]
pub(crate) fn exists(user: &str) -> bool {
    list_users().unwrap().contains(&user.to_owned())
}

pub(crate) fn is_active(user: &str) -> bool {
    match is_active_err(user) {
        Ok(res) => res,
        // Default to active
        Err(err) => {
            error!("Active checking encountered an error {err}, defaulting to active");
            true
        }
    }
}

// TODO: add Windows version
#[cfg(target_os = "linux")]
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

#[cfg(target_os = "windows")]
/// Unsafe
fn is_active_err(user: &str) -> Result<bool, Error> {
    let current_user = get_current_user()?;
    Ok(user == current_user)
}

#[cfg(target_os = "windows")]
fn get_current_user() -> Result<String, Error> {
    let mut buffer: Vec<u16> = Vec::with_capacity(256);
    let current_user = PWSTR::from_raw(buffer.as_mut_ptr());
    let mut username_len = 0;
    let username_len: *mut u32 = &mut username_len;

    let current_user = unsafe {
        match WindowsProgramming::GetUserNameW(current_user, username_len) {
            Err(_) => {
                let mut buffer =
                    Vec::<u16>::with_capacity(*username_len as usize);
                let current_user = PWSTR::from_raw(buffer.as_mut_ptr());
                WindowsProgramming::GetUserNameW(current_user, username_len)
                    .unwrap();
                current_user
            }
            Ok(_) => current_user,
        }
    };

    if current_user.is_null() {
        return Err(Error::GetUserName(
            "Windows current username returned None".to_string(),
        ));
    }

    unsafe { Ok(current_user.to_string()?) }
}
