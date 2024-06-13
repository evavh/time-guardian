use std::fmt::Display;

use log::error;

pub fn log_error<E: Display>(fallible: Result<(), E>, message: &str) {
    match fallible {
        Ok(()) => return,
        Err(err) => error!("{message}: {err}"),
    }
}
