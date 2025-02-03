use std::{fmt, process};

use tracing::error;

pub fn die(msg: &str) -> ! {
    error!("{msg}");
    process::exit(1);
}

pub fn fatal(msg: &str, err: impl fmt::Display) -> ! {
    error!("Error {msg}: {err}");
    process::exit(1);
}
