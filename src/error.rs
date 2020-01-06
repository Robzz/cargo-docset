use failure::Error as FailureError;
use snafu::Snafu;

use std::{error::Error as StdError, result::Result as StdResult};

pub struct FailureCompat {
    e: FailureError,
}

impl FailureCompat {
    /// Create a new FailureCompat.
    fn new(e: FailureError) -> FailureCompat {
        FailureCompat { e }
    }
}

impl std::fmt::Debug for FailureCompat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.e)
    }
}

impl std::fmt::Display for FailureCompat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.e)
    }
}

impl StdError for FailureCompat {}

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    #[snafu(display("Cargo error: {}", source))]
    Cargo {
        #[snafu(source(from(FailureError, FailureCompat::new)))]
        source: FailureCompat,
    },
    #[snafu(display("Cargo doc error: {}", source))]
    CargoDoc {
        #[snafu(source(from(FailureError, FailureCompat::new)))]
        source: FailureCompat,
    },
    #[snafu(display("Cargo configuration error: {}", source))]
    CargoConfig {
        #[snafu(source(from(FailureError, FailureCompat::new)))]
        source: FailureCompat,
    },
    #[snafu(display("Cargo clean error: {}", source))]
    CargoClean {
        #[snafu(source(from(FailureError, FailureCompat::new)))]
        source: FailureCompat,
    },
    #[snafu(display("Cannot determine the current directory: {}", source))]
    Cwd {
        source: std::io::Error,
    },
    #[snafu(display("I/O error: {}", source))]
    IoRead {
        source: std::io::Error,
    },
    #[snafu(display("I/O error: {}", source))]
    IoWrite {
        source: std::io::Error,
    },
    Sqlite {
        source: rusqlite::Error,
    },
    #[snafu(display("Invalid arguments: {}", msg))]
    Args {
        msg: &'static str,
    },
}

pub type Result<T> = StdResult<T, Error>;
