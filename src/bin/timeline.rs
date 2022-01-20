use calculate_differences::{
    error::TimelineError,
    opts::TimelineOpts,
    parse::{parse_tracks, parse_zones},
    query::{query_config::QueryConfig, run_query},
    zone_search::set_zone_idx,
};
use log::info;
use structopt::StructOpt;

fn main() -> Result<(), TimelineError> {
    env_logger::init();

    info!("parsing opts");

    let opts = TimelineOpts::from_args();

    info!("parsing tracks");
    let tracks = parse_tracks(&opts)?;

    info!("parsing query config");
    let query_config: QueryConfig = opts.query_file.parse()?;

    info!("parsing parse_zones");
    let mut zones = parse_zones(&opts, &tracks)?;

    info!("sorting zones");
    zones.sort_by(|a, b| a.start_time.cmp(&b.start_time));

    info!("setting zone index");
    set_zone_idx(&mut zones);

    info!("running queries: zones: {}", zones.len());
    for query in &query_config.queries {
        info!("query: {:?}", query);
        run_query(query, &query_config, &zones)?;
    }

    return Ok(());
}
