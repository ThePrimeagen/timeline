use calculate_differences::{node::{build_trees}, error::TimelineError, opts::TelemetryTimelineOpts, tm_csv::{parse_tracks}, query::{parse_query, run_query}, zone::parse_zones};
use structopt::StructOpt;

/*
fn create_time_to_out_zones(
    leaders: &Vec<Node>,
    measurement_type: String,
    from: String,
    to: String,
) -> Vec<OutZone> {
    return leaders
        .iter()
        .map(|leader| {
            return OutZone {
                measurement_type: measurement_type.clone(),
                name: format!("{} - {}", from, to),
                duration: leader.child_by_name(&from).time_to(&to),
            };
        })
        .collect();
}

fn create_self_time_out_zones(
    leaders: &Vec<Node>,
    measurement_type: String,
    name: String,
) -> Vec<OutZone> {
    return leaders
        .iter()
        .map(|leader| {
            return OutZone {
                measurement_type: measurement_type.clone(),
                name: name.clone(),
                duration: leader.calc_time(&name),
            };
        })
        .collect();
}
*/

fn main() -> Result<(), TimelineError> {
    match dotenv::dotenv() {
        _ => {}
    }

    let opts = TelemetryTimelineOpts::from_args();
    let query_config = parse_query(&opts)?;
    let main_track = parse_tracks(&opts)?.expect("You should always have a main track");
    let zones = parse_zones(&main_track, &query_config, &opts)?;
    let nodes = build_trees(zones, &query_config);

    query_config.queries
        .iter()
        .flat_map(|q| {
            return run_query(&nodes, &q, &query_config);
        })
        .for_each(|result| {
            println!("{}", result.to_string());
        });

    return Ok(());
}

