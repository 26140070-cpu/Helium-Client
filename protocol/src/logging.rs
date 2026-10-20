use std::fs::File;
use std::path::Path;

use log::LevelFilter;
use simplelog::{Config, WriteLogger};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LoggerError {
    #[error("could not create log file: {0}")]
    CreateFile(#[from] std::io::Error),

    #[error("a logger is already installed")]
    AlreadyInstalled(#[from] log::SetLoggerError),
}

pub fn init_file_logger(path: impl AsRef<Path>, level: LevelFilter) -> Result<(), LoggerError> {
    let result = (|| {
        let file = File::create(path.as_ref())?;
        WriteLogger::init(level, Config::default(), file)?;
        Ok(())
    })();
    if let Err(ref e) = result {
        eprintln!("[protocol] file logger init failed: {e}");
    }
    result
}
