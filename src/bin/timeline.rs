use calculate_differences::{opts::TimelineOpts, parse::{parse_tracks, parse_zones}, error::TimelineError, zones::set_zone_idx};
use structopt::StructOpt;


fn main() -> Result<(), TimelineError> {
    let opts = TimelineOpts::from_args();
    let tracks = parse_tracks(&opts)?;
    let mut zones = parse_zones(&opts)?;

    zones.sort();
    set_zone_idx(&mut zones);

    return Ok(());
}

