use std::{
    collections::{HashMap, HashSet},
    fs::File,
};

use serde::Deserialize;

use crate::{
    error::TimelineError,
    node::{Node, TimeCalculation},
    opts::TelemetryTimelineOpts,
};

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Query {
    Distance {
        from: String,
        to: String,
        start: bool,
        end: bool,
    },
}

#[derive(Debug, Deserialize)]
pub struct QueryConfig {
    pub root: String,
    pub zones: HashSet<String>,
    pub queries: Vec<Query>,
}

#[derive(Debug, PartialEq)]
pub struct QueryResult {
    measurement_type: String,
    name: String,
    duration: u64,
}

impl ToString for QueryResult {
    fn to_string(&self) -> String {
        return format!("{},{},{}", self.measurement_type, self.name, self.duration);
    }
}

pub fn parse_query(opts: &TelemetryTimelineOpts) -> Result<QueryConfig, TimelineError> {
    let query_file = File::open(&opts.query_file)?;
    return Ok(serde_json::de::from_reader(query_file)?);
}

fn calculate_distance(
    froms: &[&Node],
    from: &str,
    to: &str,
    side: TimeCalculation,
) -> Vec<QueryResult> {
    return froms
        .iter()
        .flat_map(|c| c.time_to(&to, &side))
        .map(|d| {
            return QueryResult {
                measurement_type: "Distance".to_string(),
                name: format!("{}-{}", from, to),
                duration: d,
            };
        })
        .collect::<Vec<QueryResult>>();
}

pub fn run_query(roots: &Vec<Node>, query: &Query) -> Vec<QueryResult> {
    // TODO: Probably should extract this.  Its... bad
    match query {
        Query::Distance {
            from,
            to,
            start,
            end,
        } => {
            let froms = roots
                .iter()
                .flat_map(|r| r.nodes_by_name(from))
                .collect::<Vec<&Node>>();

            return start
                .then(|| calculate_distance(&froms, from, to, TimeCalculation::Start))
                .unwrap_or_else(|| vec![])
                .into_iter()
                .chain(
                    end.then(|| calculate_distance(&froms, from, to, TimeCalculation::End))
                        .unwrap_or_else(|| vec![])
                        .into_iter(),
                )
                .collect::<Vec<QueryResult>>();
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{node::build_trees, zone::Zone};
    use super::*;

    #[test]
    fn test_run_query() {
        let zones = vec![
            Zone::new("A".to_string(), 50, 100),
            Zone::new("D".to_string(), 48, 50),
            Zone::new("B".to_string(), 68, 71),
            Zone::new("C".to_string(), 68, 70),
            Zone::new("A".to_string(), 101, 120),
            Zone::new("B".to_string(), 110, 112),
            Zone::new("D".to_string(), 122, 125),

            // this one has B (from the node diagram)
            Zone::new("A".to_string(), 200, 300),
            Zone::new("B".to_string(), 270, 280),
            Zone::new("C".to_string(), 230, 240),
        ];

        let query: QueryConfig = QueryConfig {
            root: "A".to_string(),
            zones: HashSet::new(),
            queries: vec![],
        };

        let roots = build_trees(zones, &query);
        let results = run_query(
            &roots,
            &Query::Distance {
                from: "A".to_string(),
                to: "B".to_string(),
                start: true,
                end: false,
            },
        );

        assert_eq!(results.len(), 3);
        assert_eq!(results.get(0).unwrap(), &QueryResult {
            measurement_type: "Distance".to_string(),
            name: "A-B".to_string(),
            duration: 68 - 50,
        });
        assert_eq!(results.get(1).unwrap(), &QueryResult {
            measurement_type: "Distance".to_string(),
            name: "A-B".to_string(),
            duration: 110 - 101,
        });
        assert_eq!(results.get(2).unwrap(), &QueryResult {
            measurement_type: "Distance".to_string(),
            name: "A-B".to_string(),
            duration: 270 - 200 - (240 - 230),
        });
    }
}
