use std::{fs::File, num::ParseIntError};

use csv::StringRecord;
use log::debug;
use serde::Deserialize;

use crate::{error::TimelineError, opts::TelemetryTimelineOpts};

#[derive(Debug, Deserialize)]
pub struct Track {
    pub id: u64,
    pub name: String,
    pub time_start: u64,
}

#[derive(Debug)]
pub struct Tracks {
    pub main_track: Option<Track>,
    pub context: Option<Track>,
}

impl Tracks {
    fn empty() -> Tracks {
        return Tracks { main_track: None, context: None };
    }
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

pub fn parse_tracks(opts: &TelemetryTimelineOpts) -> Result<Tracks, TimelineError> {
    let mut track_reader = csv::Reader::from_reader(File::open(&opts.track_file)?);
    let mut tracks: Vec<Track> = vec![];
    for result in track_reader.records() {
        debug!("reading track: {:?}", result);
        tracks.push(result?.try_into()?);
    }

    let main_track: Tracks = tracks.into_iter().fold(Tracks::empty(), |mut tracks, track| {
        debug!("checking track: {} {}", track.name, opts.context_track);
        if track.name == opts.main_track {
            tracks.main_track = Some(track);
        } else if track.name == opts.context_track {
            tracks.context = Some(track);
        }

        return tracks;
    });

    return Ok(main_track);
}
