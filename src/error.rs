use std::num::ParseIntError;

#[derive(Debug)]
pub enum TimelineError {
    ParseInt(ParseIntError),
    CsvRead(csv::Error),
    Io(std::io::Error),
    Json(serde_json::Error),
    MainTrack(String),
}

impl From<ParseIntError> for TimelineError {
    fn from(e: ParseIntError) -> Self {
        return Self::ParseInt(e);
    }
}

impl From<serde_json::Error> for TimelineError {
    fn from(e: serde_json::Error) -> Self {
        return Self::Json(e);
    }
}

impl From<csv::Error> for TimelineError {
    fn from(e: csv::Error) -> Self {
        return Self::CsvRead(e);
    }
}

impl From<std::io::Error> for TimelineError {
    fn from(e: std::io::Error) -> Self {
        return Self::Io(e);
    }
}

