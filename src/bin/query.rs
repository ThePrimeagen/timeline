use calculate_differences::{
    error::TimelineError,
    node::build_trees,
    opts::TelemetryTimelineOpts,
    query::{parse_query, run_query},
    tm_csv::parse_tracks,
    zone::parse_zones,
};
use log::debug;
use structopt::StructOpt;

fn main() -> Result<(), TimelineError> {
    env_logger::init();
    match dotenv::dotenv() {
        _ => {}
    }

    let opts = TelemetryTimelineOpts::from_args();
    let query_config = parse_query(&opts)?;
    let tracks = parse_tracks(&opts)?;
    if tracks.main_track.is_none() {
        return Err(TimelineError::MainTrack(format!(
            "Did not find the main track that you provided: {}",
            opts.main_track
        )));
    }

    debug!("track: {:?}", tracks);
    let zones = parse_zones(&tracks, &query_config, &opts)?;
    let nodes = build_trees(zones.0, &query_config);
    let contexts = zones.1;
    debug!("parsed contexts: {:?}", contexts);

    query_config
        .queries
        .iter()
        .flat_map(|q| {
            return run_query(&nodes, &q, &query_config, &contexts);
        })
        .for_each(|result| {
            println!("{}", result.to_string());
        });

    return Ok(());
}
