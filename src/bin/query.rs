use calculate_differences::{node::{build_trees}, error::TimelineError, opts::TelemetryTimelineOpts, tm_csv::{parse_tracks, Track}, query::parse_query, zone::parse_zones};
use structopt::StructOpt;

const LEADER: &str = "V8.ExternalCallback";

struct OutZone {
    measurement_type: String,
    name: String,
    duration: u64,
}

impl ToString for OutZone {
    fn to_string(&self) -> String {
        return format!("{},{},{}", self.measurement_type, self.name, self.duration);
    }
}

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
    let query = parse_query(&opts)?;
    let main_track: Track = match parse_tracks(&opts)? {
        Some(track) => track,
        None => unreachable!("You should always have a main track"),
    };

    let zones = parse_zones(&main_track, &query, &opts)?;
    let _nodes = build_trees(zones, &query);

    /*
    let leaders = next_leaders;

    create_time_to_out_zones(
        &leaders,
        "DISTANCE BETWEEN".to_string(),
        starting_event.clone(),
        bridge_method.to_string(),
    )
    .iter()
    .for_each(|zone| println!("{}", zone.to_string()));

    if !method.is_empty() {
        create_time_to_out_zones(
            &leaders,
            "DISTANCE BETWEEN".to_string(),
            bridge_method.to_string(),
            method.to_string(),
        )
        .iter()
        .for_each(|zone| println!("{}", zone.to_string()));
    }

    create_self_time_out_zones(&leaders, "SELF_TIME".to_string(), bridge_method.to_string())
        .iter()
        .for_each(|zone| println!("{}", zone.to_string()));

    if !method.is_empty() {
        create_self_time_out_zones(&leaders, "SELF_TIME".to_string(), method.to_string())
            .iter()
            .for_each(|zone| println!("{}", zone.to_string()));
    }

    create_self_time_out_zones(
        &leaders,
        "SELF_TIME".to_string(),
        starting_event.to_string(),
    )
    .iter()
    .for_each(|zone| println!("{}", zone.to_string()));
    */

    Ok(())
}

