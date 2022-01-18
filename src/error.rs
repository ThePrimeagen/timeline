use std::num::ParseIntError;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TimelineError {
    #[error("Unable to parse a record")]
    Io(#[from] std::io::Error),

    #[error("Unable to parse number...")]
    ParseIntError(#[from] ParseIntError),

    #[error("Unable to parse the csv...")]
    CsvError(#[from] csv::Error),
}

