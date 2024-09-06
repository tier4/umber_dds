use speedy;
use std::io;
use thiserror;

pub type IoResult<T> = std::result::Result<T, IoError>;

#[derive(Debug, thiserror::Error)]
pub enum IoError {
    #[error("SpeedyError: {0}")]
    SpeedyError(speedy::Error),

    #[error("{0}")]
    IoError(#[from] io::Error),
}
