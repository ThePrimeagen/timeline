use std::{collections::HashSet, fs::File};

use log::{debug, info};

use serde::Deserialize;

use crate::{
    error::TimelineError,
    node::{Node, TimeCalculation},
    opts::TelemetryTimelineOpts,
    zone::Zone,
};

#[derive(Debug, Deserialize)]
pub struct Distance {
    from: String,
    to: String,
    start: bool,
    end: bool,
    context: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct TotalTime {
    node: String,
    context: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct SelfTime {
    node: String,
    context: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct Count {
    root: String,
    child_name: String,
    context: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Query {
    Distance(Distance),
    SelfTime(SelfTime),
    TotalTime(TotalTime),
    Count(Count),
}

#[derive(Debug, Deserialize)]
pub struct QueryConfig {
    pub root: String,
    pub zones: HashSet<String>,
    pub queries: Vec<Query>,
    pub ignores: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct QueryResult {
    measurement_type: String,
    name: String,
    count: u64,
}

impl ToString for QueryResult {
    fn to_string(&self) -> String {
        return format!("{},{},{}", self.measurement_type, self.name, self.count);
    }
}

pub fn parse_query(opts: &TelemetryTimelineOpts) -> Result<QueryConfig, TimelineError> {
    let query_file = File::open(&opts.query_file)?;
    return Ok(serde_json::de::from_reader(query_file)?);
}

fn get_contexts<'a>(
    contexts: &'a Vec<Zone>,
    filter: &Option<Vec<String>>,
) -> Option<Vec<&'a Zone>> {
    let mut found_contexts: Option<Vec<&'a Zone>> = None;

    if let Some(c) = &filter {
        found_contexts = Some(
            contexts
                .iter()
                .filter(|context| {
                    return c.contains(&context.name);
                })
                .collect::<Vec<&Zone>>(),
        );
    }

    return found_contexts;
}

fn filter_nodes(node: &Node, contexts: &Option<Vec<&Zone>>) -> bool {
    if let Some(context) = &contexts {
        let mut ret = false;
        for zone in context {
            debug!(
                "context zone({:?})#contains({:?}) = {}",
                zone,
                node.zone,
                zone.contains(&node.zone)
            );

            if zone.contains(&node.zone) {
                ret = true;
                break;
            }
        }
        return ret;
    }
    return true;
}

fn calculate_distance(
    froms: &[&Node],
    distance: &Distance,
    contexts: &Option<Vec<&Zone>>,
    ignores: &[String],
    side: TimeCalculation,
) -> Vec<QueryResult> {
    return froms
        .iter()
        // TODO: Determine if this can ever actually be an option?
        .filter(|node| {
            filter_nodes(
                node.child_by_name(&distance.to)
                    .expect("this child should always exist"),
                contexts,
            )
        })
        .flat_map(|c| c.time_to(&distance.to, ignores, &side))
        .map(|d| {
            return QueryResult {
                measurement_type: "Distance".to_string(),
                name: format!("{}-{}", distance.from, distance.to),
                count: d,
            };
        })
        .collect::<Vec<QueryResult>>();
}

fn distance_query(
    roots: &Vec<Node>,
    contexts: &Vec<Zone>,
    query_config: &QueryConfig,
    distance: &Distance,
) -> Vec<QueryResult> {
    info!("distance_query");
    let found_contexts = get_contexts(contexts, &distance.context);

    debug!("found context: {:?}", found_contexts);

    let froms = roots
        .iter()
        .flat_map(|r| r.nodes_by_name(&distance.from))
        .collect::<Vec<&Node>>();

    debug!("froms: {}", froms.len());

    return distance
        .start
        .then(|| {
            calculate_distance(
                &froms,
                &distance,
                &found_contexts,
                &query_config.ignores,
                TimeCalculation::Start,
            )
        })
        .unwrap_or_else(|| vec![])
        .into_iter()
        .chain(
            distance
                .end
                .then(|| {
                    calculate_distance(
                        &froms,
                        &distance,
                        &found_contexts,
                        &query_config.ignores,
                        TimeCalculation::End,
                    )
                })
                .unwrap_or_else(|| vec![])
                .into_iter(),
        )
        .collect::<Vec<QueryResult>>();
}

fn self_time_query(
    roots: &Vec<Node>,
    contexts: &Vec<Zone>,
    self_time: &SelfTime,
) -> Vec<QueryResult> {
    let found_contexts = get_contexts(contexts, &self_time.context);

    return roots
        .iter()
        .flat_map(|r| r.nodes_by_name(&self_time.node))
        .filter(|n| filter_nodes(n, &found_contexts))
        .map(|n| {
            return QueryResult {
                measurement_type: "SelfTime".to_string(),
                name: self_time.node.clone(),
                count: n.calc_self_time(),
            };
        })
        .collect();
}

fn total_time_query(
    roots: &Vec<Node>,
    contexts: &Vec<Zone>,
    total_time: &TotalTime,
    query_config: &QueryConfig,
) -> Vec<QueryResult> {
    let found_contexts = get_contexts(contexts, &total_time.context);
    return roots
        .iter()
        .flat_map(|r| r.nodes_by_name(&total_time.node))
        .filter(|n| filter_nodes(n, &found_contexts))
        .map(|n| {
            return QueryResult {
                measurement_type: "TotalTime".to_string(),
                name: total_time.node.clone(),
                count: n.calc_total_time(&query_config.ignores),
            };
        })
        .collect();
}

fn count_query(
    roots: &Vec<Node>,
    contexts: &Vec<Zone>,
    count: &Count,
) -> Vec<QueryResult> {
    let found_contexts = get_contexts(contexts, &count.context);
    return roots
        .iter()
        .flat_map(|r| r.nodes_by_name(&count.root))
        .map(|n| {
            n.nodes_by_name(&count.child_name)
                .iter()
                .filter(|n| filter_nodes(n, &found_contexts))
                .count()
        })
        .filter(|&c| c > 0)
        .map(|c| {
            return QueryResult {
                measurement_type: "Count".to_string(),
                name: format!("{}-{}", count.root, count.child_name),
                count: c as u64,
            };
        })
        .collect();
}

pub fn run_query(
    roots: &Vec<Node>,
    query: &Query,
    query_config: &QueryConfig,
    contexts: &Vec<Zone>,
) -> Vec<QueryResult> {
    debug!(" -------------------------- run query ---------------------");
    debug!("run_query: {:?}", query);

    match query {
        Query::Distance(d) => distance_query(roots, contexts, query_config, d),
        Query::SelfTime(s) => self_time_query(roots, contexts, s),
        Query::TotalTime(t) => total_time_query(roots, contexts, t, query_config),
        Query::Count(c) => count_query(roots, contexts, c),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{node::build_trees, zone::Zone};

    fn get_zones() -> Vec<Zone> {
        return vec![
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
    }

    #[test]
    fn test_distance() {
        let _ = env_logger::builder().is_test(true).try_init();
        let zones = get_zones();
        let query_config: QueryConfig = QueryConfig {
            root: "A".to_string(),
            zones: HashSet::new(),
            queries: vec![],
            ignores: vec![],
        };

        let roots = build_trees(zones, &query_config);
        let results = run_query(
            &roots,
            &Query::Distance(Distance {
                from: "A".to_string(),
                to: "B".to_string(),
                context: None,
                start: true,
                end: false,
            }),
            &query_config,
            &vec![],
        );

        assert_eq!(results.len(), 3);
        assert_eq!(
            results.get(0).unwrap(),
            &QueryResult {
                measurement_type: "Distance".to_string(),
                name: "A-B".to_string(),
                count: 68 - 50,
            }
        );
        assert_eq!(
            results.get(1).unwrap(),
            &QueryResult {
                measurement_type: "Distance".to_string(),
                name: "A-B".to_string(),
                count: 110 - 101,
            }
        );
        assert_eq!(
            results.get(2).unwrap(),
            &QueryResult {
                measurement_type: "Distance".to_string(),
                name: "A-B".to_string(),
                count: 270 - 200 - (240 - 230),
            }
        );
    }

    #[test]
    fn test_distance_with_context() {
        let _ = env_logger::builder().is_test(true).try_init();
        let zones = get_zones();

        let query_config: QueryConfig = QueryConfig {
            root: "A".to_string(),
            zones: HashSet::new(),
            queries: vec![],
            ignores: vec![],
        };

        let roots = build_trees(zones, &query_config);
        let results = run_query(
            &roots,
            &Query::Distance(Distance {
                from: "A".to_string(),
                to: "B".to_string(),
                context: Some(vec!["Foo".to_string()]),
                start: true,
                end: false,
            }),
            &query_config,
            &vec![Zone::new("Foo".to_string(), 100, 115)],
        );

        assert_eq!(results.len(), 1);
        assert_eq!(
            results.get(0).unwrap(),
            &QueryResult {
                measurement_type: "Distance".to_string(),
                name: "A-B".to_string(),
                count: 110 - 101,
            }
        );
    }

    #[test]
    fn test_self_time() {
        let _ = env_logger::builder().is_test(true).try_init();
        let zones = get_zones();

        let query_config: QueryConfig = QueryConfig {
            root: "A".to_string(),
            zones: HashSet::new(),
            queries: vec![],
            ignores: vec![],
        };

        let roots = build_trees(zones, &query_config);
        let results = run_query(
            &roots,
            &Query::SelfTime(SelfTime {
                node: "B".to_string(),
                context: None,
            }),
            &query_config,
            &vec![],
        );

        assert_eq!(results.len(), 3);
        assert_eq!(
            results.get(0).unwrap(),
            &QueryResult {
                measurement_type: "SelfTime".to_string(),
                name: "B".to_string(),
                count: 1,
            }
        );
        assert_eq!(
            results.get(1).unwrap(),
            &QueryResult {
                measurement_type: "SelfTime".to_string(),
                name: "B".to_string(),
                count: 2,
            }
        );
        assert_eq!(
            results.get(2).unwrap(),
            &QueryResult {
                measurement_type: "SelfTime".to_string(),
                name: "B".to_string(),
                count: 10,
            }
        );
    }

    #[test]
    fn test_self_time_with_context() {
        let _ = env_logger::builder().is_test(true).try_init();
        let zones = get_zones();

        let query_config: QueryConfig = QueryConfig {
            root: "A".to_string(),
            zones: HashSet::new(),
            queries: vec![],
            ignores: vec![],
        };

        let roots = build_trees(zones, &query_config);
        let results = run_query(
            &roots,
            &Query::SelfTime(SelfTime {
                node: "B".to_string(),
                context: Some(vec!["Foo".to_string()]),
            }),
            &query_config,
            &vec![Zone::new("Foo".to_string(), 100, 115)],
        );

        assert_eq!(results.len(), 1);
        assert_eq!(
            results.get(0).unwrap(),
            &QueryResult {
                measurement_type: "SelfTime".to_string(),
                name: "B".to_string(),
                count: 2,
            }
        );
    }

    #[test]
    fn test_total_time() {
        let _ = env_logger::builder().is_test(true).try_init();
        let zones = get_zones();

        let query_config: QueryConfig = QueryConfig {
            root: "A".to_string(),
            zones: HashSet::new(),
            queries: vec![],
            ignores: vec![],
        };

        let roots = build_trees(zones, &query_config);
        let results = run_query(
            &roots,
            &Query::TotalTime(TotalTime {
                node: "A".to_string(),
                context: None,
            }),
            &query_config,
            &vec![],
        );

        assert_eq!(results.len(), 3);
        assert_eq!(
            results.get(0).unwrap(),
            &QueryResult {
                measurement_type: "TotalTime".to_string(),
                name: "A".to_string(),
                count: 50,
            }
        );
        assert_eq!(
            results.get(1).unwrap(),
            &QueryResult {
                measurement_type: "TotalTime".to_string(),
                name: "A".to_string(),
                count: 19,
            }
        );
        assert_eq!(
            results.get(2).unwrap(),
            &QueryResult {
                measurement_type: "TotalTime".to_string(),
                name: "A".to_string(),
                count: 100,
            }
        );
    }

    #[test]
    fn test_total_time_with_context() {
        let _ = env_logger::builder().is_test(true).try_init();
        let zones = get_zones();

        let query_config: QueryConfig = QueryConfig {
            root: "A".to_string(),
            zones: HashSet::new(),
            queries: vec![],
            ignores: vec![],
        };

        let roots = build_trees(zones, &query_config);
        let results = run_query(
            &roots,
            &Query::TotalTime(TotalTime {
                node: "A".to_string(),
                context: Some(vec!["Foo".to_string()]),
            }),
            &query_config,
            &vec![Zone::new("Foo".to_string(), 100, 120)],
        );

        assert_eq!(results.len(), 1);
        assert_eq!(
            results.get(0).unwrap(),
            &QueryResult {
                measurement_type: "TotalTime".to_string(),
                name: "A".to_string(),
                count: 19,
            }
        );
    }

    #[test]
    fn test_count() {
        let _ = env_logger::builder().is_test(true).try_init();
        let zones = get_zones();

        let query_config: QueryConfig = QueryConfig {
            root: "A".to_string(),
            zones: HashSet::new(),
            queries: vec![],
            ignores: vec![],
        };

        let roots = build_trees(zones, &query_config);
        let results = run_query(
            &roots,
            &Query::Count(Count {
                root: "A".to_string(),
                child_name: "B".to_string(),
                context: None
            }),
            &query_config,
            &vec![],
        );

        assert_eq!(results.len(), 3);
        assert_eq!(
            results.get(0).unwrap(),
            &QueryResult {
                measurement_type: "Count".to_string(),
                name: "A-B".to_string(),
                count: 1,
            }
        );
        assert_eq!(
            results.get(1).unwrap(),
            &QueryResult {
                measurement_type: "Count".to_string(),
                name: "A-B".to_string(),
                count: 1,
            }
        );
        assert_eq!(
            results.get(2).unwrap(),
            &QueryResult {
                measurement_type: "Count".to_string(),
                name: "A-B".to_string(),
                count: 1,
            }
        );
    }

    #[test]
    fn test_count_with_context() {
        let _ = env_logger::builder().is_test(true).try_init();
        let zones = get_zones();

        let query_config: QueryConfig = QueryConfig {
            root: "A".to_string(),
            zones: HashSet::new(),
            queries: vec![],
            ignores: vec![],
        };

        let roots = build_trees(zones, &query_config);
        let results = run_query(
            &roots,
            &Query::Count(Count {
                root: "A".to_string(),
                child_name: "B".to_string(),
                context: Some(vec!["Foo".to_string()]),
            }),
            &query_config,
            &vec![Zone::new("Foo".to_string(), 260, 280)],
        );

        assert_eq!(results.len(), 1);
        assert_eq!(
            results.get(0).unwrap(),
            &QueryResult {
                measurement_type: "Count".to_string(),
                name: "A-B".to_string(),
                count: 1,
            }
        );
    }
}
