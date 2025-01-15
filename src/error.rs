use std::process;

use thiserror::Error;

pub type RemindmeResult<T> = Result<T, RemindmeError>;

#[derive(Error, Debug)]
pub enum RemindmeError {
    #[cfg(feature = "client")]
    #[error("unable to parse time string \"{0}\"")]
    ParseTime(String),

    #[cfg(feature = "client")]
    #[error("time delta out of bounds \"{0} seconds\"")]
    TimeDeltaSec(u64),

    #[cfg(feature = "client")]
    #[error("time delta out of bounds \"{0} minutes\"")]
    TimeDeltaMin(u64),

    #[cfg(feature = "client")]
    #[error("time delta out of bounds \"{0} hours\"")]
    TimeDeltaH(u64),

    #[cfg(feature = "client")]
    #[error("time delta out of bounds \"{0} days\"")]
    TimeDeltaD(u64),

    #[cfg(feature = "client")]
    #[error("time delta out of bounds \"{0} weeks\"")]
    TimeDeltaW(u64),

    #[cfg(feature = "client")]
    #[error("request error - {0}")]
    Reqwest(#[from] reqwest::Error),

    #[cfg(feature = "client")]
    #[error("command exited with status {0}")]
    CommandExit(i32),

    #[error("io error - {0}")]
    IO(#[from] std::io::Error),
}

pub fn resolve<T>(result: RemindmeResult<T>) -> T {
    match result {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        },
    }
}
