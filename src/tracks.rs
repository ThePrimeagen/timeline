use csv::StringRecord;

pub struct Track {
    name: String,
    id: usize,
}

impl TryInto<Track> for StringRecord {
    type Error = std::num::ParseIntError;
    fn try_into(self) -> Result<Track, Self::Error> {
        return Ok(Track {
            name: self[2].to_string(),
            id: self[1].parse()?,
        });
    }
}

