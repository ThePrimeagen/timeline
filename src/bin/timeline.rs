use calculate_differences::{opts::TimelineOpts, parse::{parse_tracks, parse_zones}, error::TimelineError};
use structopt::StructOpt;


fn main() -> Result<(), TimelineError> {
    let opts = TimelineOpts::from_args();
    let tracks = parse_tracks(&opts)?;
    let mut zones = parse_zones(&opts)?;

    zones.sort();

    return Ok(());
}

