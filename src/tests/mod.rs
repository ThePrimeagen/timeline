use crate::zones::Zone;

pub trait TestZone {
    fn from_timestamps(start_time: u64, end_time: u64) -> Zone;
    fn from_csv_strings<'a>(strings: Vec<&'a str>) -> Result<Vec<Zone>, std::num::ParseIntError>;
}

impl TestZone for Zone {
    fn from_timestamps(start_time: u64, end_time: u64) -> Zone {
        return Zone::new("foo".to_string(), start_time, end_time, 0);
    }

    // This is for when I do a Reduce query to create a test
    // TM_ZONE,65536,V8.Builtin_HandleApiCall,1642630333023839586,1642630333023880460,6,0x0,0x0,/home/mpaulson/.pvm/installed/odroid-x86_64/32-release-21.2/dev/.pvm/build/src/src/base/Telemetry.h,111,0,0
    fn from_csv_strings(strings: Vec<&str>) -> Result<Vec<Zone>, std::num::ParseIntError> {
        let mut out = vec![];

        for string in strings {
            let record = string.split(",").collect::<Vec<&str>>();
            let track_id = record.get(1).unwrap().parse::<usize>()?;
            let name = record.get(2).unwrap();
            let start_time = record.get(3).unwrap().parse::<u64>()?;
            let end_time = record.get(4).unwrap().parse::<u64>()?;

            out.push(Zone::new(name.to_string(), start_time, end_time, track_id));
        }

        return Ok(out);
    }
}
