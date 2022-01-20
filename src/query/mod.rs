pub mod query_config;

use std::{fmt::Display, fs::File, io::BufReader, str::FromStr};

use log::info;
use serde::Deserialize;

use crate::{
    error::TimelineError,
    zone_search::{
        contains_intersect, filter_by_name, filter_by_name_on_idx, filter_out_contains,
        get_by_name, partial_intersect, sum_zone_indices,
    },
    zones::Zone,
};

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

#[derive(Debug, Deserialize)]
pub struct QueryConfig {
    pub ignores: Vec<String>,
    pub queries: Vec<Query>,
}

impl FromStr for QueryConfig {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let file = File::open(s)?;
        let reader = BufReader::new(file);
        return Ok(serde_json::from_reader(reader)?);
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct DataPoint {
    query: String,
    name: String,
    count: u64,
    additional_data: Option<String>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct StatResult {
    name: String,
    start_time: u64,
    end_time: u64,
    duration: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CostResult {
    name: String,
    cpp_duration: u64,
    cost_of_javascript: u64,
    cost_of_generated_bridge_method: u64,
    cost_of_args: u64,
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

fn index_to_original_csv(zones: &Vec<Zone>, idx: usize) -> QueryResult {
    return QueryResult::OriginalCsvRow(
        zones
            .get(idx)
            .expect("all indices should be valid")
            .original_csv
            .to_string(),
    );
}

fn self_time_query(query: &SelfTime, config: &QueryConfig, zones: &Vec<Zone>) -> Vec<QueryResult> {
    return get_by_name(zones, query.node.as_str())
        .iter()
        .flat_map(|z_idx| {
            return zones.get(*z_idx);
        })
        .map(|z| {
            let partials = filter_by_name_on_idx(
                zones,
                &partial_intersect(zones, z.idx),
                &query.partial_ignore,
            );

            // TODO: Should I filter out contains with contains...?
            let contains =
                filter_by_name_on_idx(zones, &contains_intersect(zones, z.idx), &config.ignores);

            let contains = filter_out_contains(zones, &partials, &contains);

            // TODO: filter out sub contains within contains

            let partials = sum_zone_indices(zones, &z, &partials);
            let contains = sum_zone_indices(zones, &z, &contains);

            return QueryResult::DataPoint(DataPoint {
                query: "SelfTime".to_string(),
                name: z.name.clone(),
                count: z.duration.saturating_sub(partials).saturating_sub(contains),
                additional_data: None,
            });
        })
        .collect::<Vec<QueryResult>>();
}

pub fn reduce_query(query: &Reduce, zones: &Vec<Zone>) -> Vec<QueryResult> {
    let names = vec![query.node.clone()];
    info!("reduce_query#filte_by_name: {:?}", names);

    let found_idxs = filter_by_name(zones, &names);
    info!("found indices: {:?}", found_idxs.len());

    let found = *found_idxs
        .iter()
        .nth(query.ignore_count.unwrap_or(0))
        .expect("to always find a reduce result");

    let found = zones.get(found).expect("all indices should be valid");

    let partials = partial_intersect(zones, found.idx);
    let contains = contains_intersect(zones, found.idx);

    let mut out = partials
        .iter()
        .map(|p| index_to_original_csv(zones, *p))
        .collect::<Vec<QueryResult>>();

    out.append(
        &mut contains
            .iter()
            .map(|p| index_to_original_csv(zones, *p))
            .collect::<Vec<QueryResult>>(),
    );

    out.push(QueryResult::OriginalCsvRow(found.original_csv.to_string()));

    return out;
}

fn stat_query(stat: &Stat, zones: &Vec<Zone>) -> Vec<QueryResult> {
    return filter_by_name(zones, &vec![stat.node.clone()])
        .iter()
        .map(|z_idx| zones.get(*z_idx).expect("all indices should be valid"))
        .map(|z| {
            return QueryResult::Stat(StatResult {
                name: z.name.clone(),
                duration: z.duration,
                start_time: z.start_time,
                end_time: z.end_time,
            });
        })
        .collect::<Vec<QueryResult>>();
}

fn cost_query(cost: &Cost, config: &QueryConfig, zones: &Vec<Zone>) -> Vec<QueryResult> {
    let out = vec![];

    return out;
}

pub fn run_query(
    query: &Query,
    config: &QueryConfig,
    zones: &Vec<Zone>,
) -> Result<(), TimelineError> {
    let results = match query {
        Query::SelfTime(s) => self_time_query(&s, config, zones),
        Query::Reduce(r) => reduce_query(&r, zones),
        Query::Stat(s) => stat_query(&s, zones),
        Query::Cost(c) => cost_query(&c, config, zones),
    };

    for result in results {
        println!("{}", result);
    }

    return Ok(());
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::zone_search::set_zone_idx;

    #[test]
    fn test_self_time_query() {
        let mut zones = vec![
            Zone::new("foo".to_string(), 8, 20, 0),
            Zone::new("foo2".to_string(), 10, 50, 0),
            Zone::new("foo".to_string(), 30, 40, 0),
            Zone::new("foo".to_string(), 48, 55, 0),
        ];

        set_zone_idx(&mut zones);

        let self_time = SelfTime {
            partial_ignore: vec!["foo".to_string()],
            node: "foo2".to_string(),
        };

        let config = QueryConfig {
            ignores: vec![],
            queries: vec![],
        };

        let res = self_time_query(&self_time, &config, &zones);

        assert_eq!(res.len(), 1);
        assert_eq!(
            *res.get(0).unwrap(),
            QueryResult::DataPoint(DataPoint {
                query: "SelfTime".to_string(),
                name: "foo2".to_string(),
                count: 28,
                additional_data: None,
            })
        )
    }

    #[test]
    fn test_self_time_query_with_ignores() {
        let mut zones = vec![
            Zone::new("foo".to_string(), 8, 20, 0),
            Zone::new("foo2".to_string(), 10, 50, 0),
            Zone::new("ignore-me".to_string(), 25, 30, 0),
            Zone::new("foo".to_string(), 30, 40, 0),
            Zone::new("foo".to_string(), 48, 55, 0),
        ];

        set_zone_idx(&mut zones);

        let self_time = SelfTime {
            partial_ignore: vec!["foo".to_string()],
            node: "foo2".to_string(),
        };

        let config = QueryConfig {
            ignores: vec!["ignore-me".to_string()],
            queries: vec![],
        };

        let res = self_time_query(&self_time, &config, &zones);

        assert_eq!(res.len(), 1);
        assert_eq!(
            *res.get(0).unwrap(),
            QueryResult::DataPoint(DataPoint {
                query: "SelfTime".to_string(),
                name: "foo2".to_string(),
                count: 23,
                additional_data: None,
            })
        )
    }

    #[test]
    fn test_reduce_query() {
        let mut zones = vec![
            Zone::new("excluded_outside_left".to_string(), 6, 20, 0),
            Zone::new("included_partial_left".to_string(), 8, 26, 0),
            Zone::new("excluded_container".to_string(), 10, 51, 0),
            Zone::new("root".to_string(), 25, 50, 0),
            Zone::new("contained".to_string(), 30, 40, 0),
            Zone::new("included_partial_right".to_string(), 48, 55, 0),
            Zone::new("excluded_outside_right".to_string(), 51, 60, 0),
        ]
        .into_iter()
        .map(|mut zone| {
            zone.original_csv = format!("{}", zone.name);
            return zone;
        })
        .collect::<Vec<Zone>>();

        set_zone_idx(&mut zones);

        let reduce = Reduce {
            node: "root".to_string(),
            ignore_count: Some(0),
        };

        let res = reduce_query(&reduce, &zones)
            .into_iter()
            .map(|qr| {
                return match qr {
                    QueryResult::OriginalCsvRow(s) => s,
                    _ => "YOU SUCK".to_string(),
                };
            })
            .collect::<Vec<String>>();

        assert_eq!(res.len(), 4);
        assert_eq!(res.get(0).unwrap(), "included_partial_left"); // left
        assert_eq!(res.get(1).unwrap(), "included_partial_right"); // right
        assert_eq!(res.get(2).unwrap(), "contained"); // right
        assert_eq!(res.get(3).unwrap(), "root"); // right
    }

    #[test]
    fn test_stat_query() {
        let mut zones = vec![
            Zone::new("foo".to_string(), 6, 20, 0),
            Zone::new("foo".to_string(), 8, 26, 0),
            Zone::new("foo2".to_string(), 10, 51, 0),
            Zone::new("foo3".to_string(), 11, 51, 0),
        ]
        .into_iter()
        .map(|mut zone| {
            zone.original_csv = format!("{}", zone.name);
            return zone;
        })
        .collect::<Vec<Zone>>();

        set_zone_idx(&mut zones);

        let stat = Stat {
            node: "foo".to_string(),
        };

        let res = stat_query(&stat, &zones)
            .into_iter()
            .map(|qr| {
                return format!("{}", qr);
            })
            .collect::<Vec<String>>();

        assert_eq!(res.len(), 2);
        assert_eq!(res.get(0).unwrap(), "foo,14,6,20"); // left
        assert_eq!(res.get(1).unwrap(), "foo,18,8,26"); // left
    }
}
