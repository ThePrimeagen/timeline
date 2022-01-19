use std::collections::HashMap;

use calculate_differences::{error::TimelineError, opts::TimelineOpts, parse::{parse_tracks, parse_zones}, zone_search::{ZoneIdx, set_zone_idx}, query::{QueryConfig, run_query}};
use structopt::StructOpt;


fn main() -> Result<(), TimelineError> {
    let opts = TimelineOpts::from_args();
    let tracks = parse_tracks(&opts)?;
    let query_config: QueryConfig = opts.query_file.parse()?;
    let mut zones = parse_zones(&opts, &tracks)?;

    zones.sort();
    set_zone_idx(&mut zones);

    for query in &query_config.queries {
        run_query(query, &query_config, &zones)?;
    }

    return Ok(());
}

