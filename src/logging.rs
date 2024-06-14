use std::fmt::Debug;

use log::error;

pub fn log_error<E: Debug>(fallible: Result<(), E>, message: &str) {
    match fallible {
        Ok(()) => return,
        Err(err) => error!("{message}: {err:?}"),
    }
}

pub fn _unwrap_or_log_else<T, E: Debug, F: FnOnce() -> T>(
    fallible: Result<T, E>,
    message: &str,
    function: F,
) -> T {
    match fallible {
        Ok(res) => res,
        Err(err) => {
            error!("{message}: {err:?}");
            function()
        }
    }
}

pub fn _unwrap_or_err_else<T, E: Debug, F: FnOnce(E) -> T>(
    fallible: Result<T, E>,
    function: F,
) -> T {
    match fallible {
        Ok(res) => res,
        Err(err) => {
            function(err)
        }
    }
}
