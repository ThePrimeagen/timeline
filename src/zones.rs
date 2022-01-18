use std::{fmt::Display, num::ParseIntError};

use csv::StringRecord;

#[derive(Debug, PartialEq, Eq)]
pub struct Zone {
    name: String,
    start_time: u64,
    end_time: u64,
    duration: u64,
    idx: usize,
}

impl Ord for Zone {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        return self.start_time.cmp(&other.start_time);
    }
}

impl PartialOrd for Zone {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        return self.start_time.partial_cmp(&other.start_time);
    }
}

impl Zone {
    pub fn new(name: String, start_time: u64, end_time: u64) -> Zone {
        return Zone {
            name,
            start_time,
            end_time,
            duration: start_time.abs_diff(end_time),
            idx: 0,
        };
    }

    pub fn starts_before(&self, zone: &Zone) -> bool {
        return self.start_time < zone.start_time;
    }

    pub fn is_before(&self, zone: &Zone) -> bool {
        return zone.start_time > self.start_time;
    }

    pub fn contains(&self, zone: &Zone) -> bool {
        return self.start_time <= zone.start_time && self.end_time >= zone.end_time;
    }

    pub fn partial_contains(&self, zone: &Zone) -> bool {
        return self.start_time > zone.start_time && self.start_time <= zone.end_time
            || self.end_time < zone.end_time && self.end_time >= zone.start_time;
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
    type Error = ParseIntError;

    fn try_into(self) -> Result<Zone, Self::Error> {
        return Ok(Zone::new(
            self[2].to_string(),
            self[3].parse()?,
            self[4].parse()?,
        ));
    }
}

impl Display for Zone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "Zone({}): {},{},{}",
            self.duration, self.name, self.start_time, self.end_time
        );
    }
}

pub fn set_zone_idx(vec: &mut Vec<Zone>) {
    vec.iter_mut().enumerate().for_each(|(idx, z)| {
        z.idx = idx;
    });
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_duration_intersection() {
        let a = Zone::new("foo".to_string(), 10, 50);
        let b = Zone::new("foo".to_string(), 8, 20);
        let c = Zone::new("foo".to_string(), 48, 55);
        let d = Zone::new("foo".to_string(), 30, 40);
        let e = Zone::new("foo".to_string(), 55, 65);

        assert_eq!(a.get_duration_intersection(&b), 10);
        assert_eq!(a.get_duration_intersection(&c), 2);
        assert_eq!(a.get_duration_intersection(&d), 10);
        assert_eq!(a.get_duration_intersection(&e), 0);
    }

    #[test]
    fn test_contains() {
        let a = Zone::new("foo".to_string(), 10, 50);
        let b = Zone::new("foo".to_string(), 8, 20);
        let c = Zone::new("foo".to_string(), 48, 55);
        let d = Zone::new("foo".to_string(), 30, 40);
        let e = Zone::new("foo".to_string(), 55, 65);

        assert_eq!(a.contains(&b), false);
        assert_eq!(a.contains(&c), false);
        assert_eq!(a.contains(&d), true);
        assert_eq!(a.contains(&e), false);
    }
}
