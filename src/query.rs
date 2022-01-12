use std::{fs::File, collections::{HashSet, HashMap}};

use serde::Deserialize;

use crate::{opts::TelemetryTimelineOpts, error::TimelineError};

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Query {
    Distance {from: String, to: String, start: bool, end: bool },
}

#[derive(Debug, Deserialize)]
pub struct QueryConfig {
    pub root: String,
    pub zones: HashSet<String>,
    pub queries: HashMap<String, Query>,
}

pub fn parse_query(opts: &TelemetryTimelineOpts) -> Result<QueryConfig, TimelineError> {
    let query_file = File::open(&opts.query_file)?;
    return Ok(serde_json::de::from_reader(query_file)?);
}

/*
pub fn run_query(zones: &Vec<Zone>) -> Result<(), TimelineError> {
}
*/

