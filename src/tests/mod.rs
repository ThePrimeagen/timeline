use crate::zones::Zone;

pub trait TestZone {
    fn from_timestamps(start_time: u64, end_time: u64) -> Zone;
}

impl TestZone for Zone {
    fn from_timestamps(start_time: u64, end_time: u64) -> Zone {
        return Zone::new("foo".to_string(), start_time, end_time, 0);
    }
}
