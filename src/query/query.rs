use std::fmt::Display;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Stat {
    pub node: String,
}

#[derive(Debug, Deserialize)]
pub struct Reduce {
    pub node: String,
    pub ignore_count: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct Cost {
    pub node: String,
}

#[derive(Debug, Deserialize)]
pub struct SelfTime {
    pub node: String,
    pub partial_ignore: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Query {
    SelfTime(SelfTime),
    Reduce(Reduce),
    Stat(Stat),
    Cost(Cost),
}

#[derive(Debug, Eq, PartialEq)]
pub struct DataPoint {
    pub query: String,
    pub name: String,
    pub count: u64,
    pub additional_data: Option<String>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct StatResult {
    pub name: String,
    pub start_time: u64,
    pub end_time: u64,
    pub duration: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CostResult {
    pub name: String,
    pub cpp_duration: u64,
    pub cost_of_javascript: u64,
    pub cost_of_generated_bridge_method: u64,
    pub cost_of_args: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub enum QueryResult {
    DataPoint(DataPoint),
    OriginalCsvRow(String),
    Stat(StatResult),
    Cost(CostResult),
}

impl Display for QueryResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryResult::DataPoint(p) => {
                return write!(
                    f,
                    "{},{},{},{}",
                    p.query,
                    p.name,
                    p.count,
                    p.additional_data.as_ref().unwrap_or(&"".to_string())
                );
            }
            QueryResult::OriginalCsvRow(s) => {
                return write!(f, "{}", s);
            }

            QueryResult::Stat(s) => {
                return write!(
                    f,
                    "{},{},{},{}",
                    &s.name, s.duration, s.start_time, s.end_time
                );
            }

            QueryResult::Cost(c) => {
                return write!(
                    f,
                    "{},{},{},{},{}",
                    &c.name,
                    c.cpp_duration,
                    c.cost_of_javascript,
                    c.cost_of_generated_bridge_method,
                    c.cost_of_args
                );
            }
        }
    }
}


