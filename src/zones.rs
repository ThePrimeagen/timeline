use std::fmt::Display;

use csv::StringRecord;
use itertools::Itertools;

use crate::error::TimelineError;

#[derive(Debug, PartialEq, Eq)]
pub struct Zone {
    pub name: String,
    pub start_time: u64,
    pub end_time: u64,
    pub duration: u64,
    pub idx: usize,
    pub track_id: usize,
    pub original_csv: String,
}

impl Zone {
    pub fn from_record(record: &StringRecord) -> Result<Self, TimelineError> {
        let mut zone = Zone::new(
            record[2].to_string(),
            record[3].parse()?,
            record[4].parse()?,
            record[1].parse()?,
        );
        zone.original_csv = record.iter().join(",");
        return Ok(zone);
    }

    pub fn new(name: String, start_time: u64, end_time: u64, track_id: usize) -> Zone {
        return Zone {
            name,
            start_time,
            end_time,
            duration: start_time.abs_diff(end_time),
            idx: 0,
            track_id,
            original_csv: "".to_string(),
        };
    }

    pub fn starts_before(&self, zone: &Zone) -> bool {
        return self.start_time < zone.start_time;
    }

    pub fn completes_before(&self, zone: &Zone) -> bool {
        return zone.start_time > self.end_time;
    }

    pub fn contains(&self, zone: &Zone) -> bool {
        return self.start_time <= zone.start_time && self.end_time >= zone.end_time;
    }

    pub fn partial_contains(&self, zone: &Zone) -> bool {
        return self.start_time > zone.start_time
            && self.start_time <= zone.end_time
            && self.end_time >= zone.end_time
            || self.end_time < zone.end_time
                && self.end_time >= zone.start_time
                && self.start_time <= zone.start_time;
    }

    pub fn get_duration_intersection(&self, zone: &Zone) -> u64 {
        let s = self.start_time.abs_diff(zone.start_time);
        let e = self.end_time.abs_diff(zone.end_time);

        return zone
            .end_time
            .max(self.end_time)
            .saturating_sub(zone.start_time.min(self.start_time))
            .saturating_sub(s + e);
    }
}

impl TryInto<Zone> for StringRecord {
    type Error = TimelineError;

    fn try_into(self) -> Result<Zone, Self::Error> {
        return Ok(Zone::from_record(&self)?);
    }
}

impl Display for Zone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "{}: Zone({}): {},{},{}",
            self.track_id, self.duration, self.name, self.start_time, self.end_time
        );
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tests::TestZone;

    #[test]
    fn test_duration_intersection() {
        let a = Zone::from_timestamps(10, 50);
        let b = Zone::from_timestamps(8, 20);
        let c = Zone::from_timestamps(48, 55);
        let d = Zone::from_timestamps(30, 40);
        let e = Zone::from_timestamps(55, 65);

        assert_eq!(a.get_duration_intersection(&b), 10);
        assert_eq!(a.get_duration_intersection(&c), 2);
        assert_eq!(a.get_duration_intersection(&d), 10);
        assert_eq!(a.get_duration_intersection(&e), 0);
    }

    #[test]
    fn test_contains() {
        let a = Zone::from_timestamps(10, 50);
        let b = Zone::from_timestamps(8, 20);
        let c = Zone::from_timestamps(48, 55);
        let d = Zone::from_timestamps(30, 40);
        let e = Zone::from_timestamps(55, 65);

        assert_eq!(a.contains(&b), false);
        assert_eq!(a.contains(&c), false);
        assert_eq!(a.contains(&d), true);
        assert_eq!(a.contains(&e), false);
    }
}
