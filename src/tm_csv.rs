use std::{fs::File, num::ParseIntError};

use csv::StringRecord;
use serde::Deserialize;

use crate::{error::TimelineError, opts::TelemetryTimelineOpts};

#[derive(Debug)]
pub enum Entry {
    Track(Track),
}

#[derive(Debug, Deserialize)]
pub struct Track {
    pub id: u64,
    pub name: String,
    pub time_start: u64,
}

impl TryFrom<StringRecord> for Track {
    type Error = ParseIntError;
    fn try_from(s: StringRecord) -> Result<Self, Self::Error> {
        return Ok(Track {
            id: s[1].parse()?,
            name: s[2].to_string(),
            time_start: s[3].parse()?,
        });
    }
}

pub fn parse_tracks(opts: &TelemetryTimelineOpts) -> Result<Option<Track>, TimelineError> {
    let mut track_reader = csv::Reader::from_reader(File::open(&opts.track_file)?);
    let mut tracks: Vec<Track> = vec![];
    for result in track_reader.records() {
        tracks.push(result?.try_into()?);
    }

    let main_track: Option<Track> = tracks.into_iter().fold(None, |curr, track| {
        if let Some(x) = curr {
            return Some(x);
        }

        if track.name == "Main Thread" {
            return Some(track);
        }

        return None;
    });

    return Ok(main_track);
}
