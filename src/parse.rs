use std::fs::File;

use log::info;

use crate::{opts::TimelineOpts, error::TimelineError, tracks::Track, zones::Zone};

pub fn parse_tracks(opts: &TimelineOpts) -> Result<Vec<Track>, TimelineError> {
    info!("about to parse tracks for file {}", opts.track_file);
    let mut track_reader = csv::Reader::from_reader(File::open(&opts.track_file)?);

    let mut tracks: Vec<Track> = vec![];
    for result in track_reader.records() {
        tracks.push(result?.try_into()?);
    }

    return Ok(tracks);
}

pub fn parse_zones(opts: &TimelineOpts, allowed_tracks: &[Track]) -> Result<Vec<Zone>, TimelineError> {
    info!("about to parse zones for file {}", opts.zone_file);
    let mut track_reader = csv::Reader::from_reader(File::open(&opts.zone_file)?);

    let mut zones: Vec<Zone> = vec![];
    for result in track_reader.records() {
        let result = result?;
        let id = result[1].parse::<usize>()?;

        if allowed_tracks.iter().any(|track| track.id == id) {
            zones.push(result.try_into()?);
        }
    }

    return Ok(zones);
}
