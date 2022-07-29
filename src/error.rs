use snafu::Snafu;

use std::result::Result as StdResult;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(display("Cargo doc exited with code {:?}.", code))]
    CargoDoc {
        code: Option<i32>
    },
    #[snafu(display("Cargo clean exited with code {:?}.", code))]
    CargoClean {
        code: Option<i32>
    },
    #[snafu(display("Error running 'cargo metadata' command: {}", source))]
    CargoMetadata {
        source: cargo_metadata::Error
    },
    #[snafu(display("Process spawn error: {}", source))]
    Spawn {
        source: std::io::Error
    },
    #[snafu(display("Cannot determine the current directory: {}", source))]
    Cwd {
        source: std::io::Error
    },
    #[snafu(display("I/O read error: {}", source))]
    IoRead {
        source: std::io::Error
    },
    #[snafu(display("I/O write error: {}", source))]
    IoWrite {
        source: std::io::Error
    },
    Sqlite {
        source: rusqlite::Error
    },
    Args {
        msg: &'static str
    }
}

pub type Result<T> = StdResult<T, Error>;
