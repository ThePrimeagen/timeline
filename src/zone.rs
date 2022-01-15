use std::{num::ParseIntError, fs::File, hash::Hash};

use csv::StringRecord;
use log::debug;

use crate::{opts::TelemetryTimelineOpts, tm_csv::{Track, Tracks}, error::TimelineError, query::QueryConfig};

#[derive(Eq, Debug)]
pub struct Zone {
    pub name: String,
    pub time_start: u64,
    pub time_end: u64,
    pub duration: u64,
}

impl Hash for Zone {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.time_start.hash(state);
        self.time_end.hash(state);
    }
}

impl TryFrom<StringRecord> for Zone {
    type Error = ParseIntError;
    fn try_from(s: StringRecord) -> Result<Self, Self::Error> {
        let ts = s[3].parse()?;
        let te = s[4].parse()?;
        return Ok(Zone::new(s[2].to_string(), ts, te));
    }
}

impl PartialOrd for Zone {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        return Some(self.time_start.cmp(&other.time_start));
    }
}

impl PartialEq for Zone {
    fn eq(&self, other: &Self) -> bool {
        return self.time_start == other.time_start &&
            self.time_end == other.time_end &&
            self.name == other.name;
    }
}

impl Ord for Zone {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        return self.time_start.cmp(&other.time_start);
    }
}

impl Zone {
    pub fn new(name: String, time_start: u64, time_end: u64) -> Zone {
        debug!("Zone::new {} {} {} {}", name, time_start, time_end, time_end - time_start);
        return Zone {
            name,
            time_start,
            time_end,
            duration: time_end - time_start,
        }
    }
    pub fn contains(&self, zone: &Zone) -> bool {
        return self.time_start <= zone.time_start && self.time_end >= zone.time_end;
    }

    pub fn ends_after(&self, zone: &Zone) -> bool {
        return self.time_end > zone.time_end;
    }

    pub fn completes_by(&self, zone: &Zone) -> bool {
        return self.time_end < zone.time_start;
    }

    pub fn get_start_distance(&self, zone: &Zone) -> u64 {
        return zone.time_start.abs_diff(self.time_start);
    }

    pub fn get_end_distance(&self, zone: &Zone) -> u64 {
        return zone.time_end.abs_diff(self.time_end);
    }
}

pub fn parse_zones(tracks: &Tracks, query: &QueryConfig, opts: &TelemetryTimelineOpts) -> Result<(Vec<Zone>, Vec<Zone>), TimelineError> {
    let mut data_reader = csv::Reader::from_reader(File::open(&opts.zone_file)?);
    let mut main_track = vec![];
    let mut context = vec![];

    for result in data_reader.records().flat_map(|r| r) {
        let id: u64 = result[1].parse()?;

        if let Some(c) = &tracks.context {
            if id == c.id {
                let zone: Zone = result.try_into()?;
                context.push(zone);
                continue;
            }
        }

        if id == tracks.main_track.as_ref().expect("main track to exist").id {
            let zone: Zone = result.try_into()?;
            debug!("parse_zones for main_track: {} {}", zone.name, query.zones.contains(&zone.name));
            if query.zones.contains(&zone.name) {
                main_track.push(zone);
            }
        }
    }

    main_track.sort();
    context.sort();
    return Ok((main_track, context));
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new() {
        let zone = Zone::new("foo".to_string(), 0, 100);
        assert_eq!(zone.duration, 100);
    }

    #[test]
    fn contains() {
        let zone = Zone::new("foo".to_string(), 50, 100);
        let zone2 = Zone::new("foo".to_string(), 68, 71);
        let zone3 = Zone::new("foo".to_string(), 68, 101);
        let zone4 = Zone::new("foo".to_string(), 49, 100);
        let zone5 = Zone::new("foo".to_string(), 49, 101);

        assert_eq!(zone.contains(&zone), true);
        assert_eq!(zone.contains(&zone2), true);

        assert_eq!(zone.contains(&zone3), false);
        assert_eq!(zone.contains(&zone4), false);
        assert_eq!(zone.contains(&zone5), false);
    }

    #[test]
    fn ends_after() {
        let zone = Zone::new("foo".to_string(), 50, 100);
        let zone2 = Zone::new("foo".to_string(), 68, 71);
        let zone3 = Zone::new("foo".to_string(), 68, 101);
        let zone4 = Zone::new("foo".to_string(), 101, 102);

        assert_eq!(zone.ends_after(&zone), false);
        assert_eq!(zone.ends_after(&zone2), true);
        assert_eq!(zone.ends_after(&zone3), false);

        assert_eq!(zone.completes_by(&zone), false);
        assert_eq!(zone.completes_by(&zone2), false);
        assert_eq!(zone.completes_by(&zone3), false);
        assert_eq!(zone.completes_by(&zone4), true);
    }

    #[test]
    fn get_start_distance() {
        let zone = Zone::new("foo".to_string(), 50, 100);
        let zone2 = Zone::new("foo".to_string(), 68, 71);
        let zone3 = Zone::new("foo".to_string(), 68, 101);

        assert_eq!(zone.get_start_distance(&zone), 0);
        assert_eq!(zone.get_start_distance(&zone2), 18);
        assert_eq!(zone.get_start_distance(&zone3), 18);
    }

    #[test]
    pub fn get_end_distance() {
        let zone = Zone::new("foo".to_string(), 50, 100);
        let zone2 = Zone::new("foo".to_string(), 68, 71);
        let zone3 = Zone::new("foo".to_string(), 68, 101);

        assert_eq!(zone.get_end_distance(&zone), 0);
        assert_eq!(zone.get_end_distance(&zone2), 29);
        assert_eq!(zone.get_end_distance(&zone3), 1);
    }
}
