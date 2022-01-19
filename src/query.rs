use std::{fmt::Display, rc::Rc, str::FromStr};

use serde::Deserialize;

use crate::{
    error::TimelineError,
    zone_search::{get_by_name, partial_intersect, contains_intersect, filter_by_name_on_idx, filter_out_contains, sum_zone_indices},
    zones::Zone,
};

#[derive(Debug, Deserialize)]
pub struct SelfTime {
    pub node: String,
    pub partial_ignore: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Query {
    SelfTime(SelfTime),
}

#[derive(Debug, Deserialize)]
pub struct QueryConfigParsed {
    pub ignores: Vec<String>,
    pub queries: Vec<Query>,
}

#[derive(Debug)]
pub struct QueryConfig {
    pub ignores: Rc<Vec<String>>,
    pub queries: Vec<Query>,
}

impl Into<QueryConfig> for QueryConfigParsed {
    fn into(self) -> QueryConfig {
        return QueryConfig {
            ignores: Rc::new(self.ignores),
            queries: self.queries,
        };
    }
}

impl FromStr for QueryConfig {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parsed: QueryConfigParsed = serde_json::from_str::<QueryConfigParsed>(s)?;
        return Ok(parsed.into());
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct QueryResult {
    query: String,
    name: String,
    count: u64,
    additional_data: Option<String>,
}

impl Display for QueryResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "{},{},{},{}",
            self.query,
            self.name,
            self.count,
            self.additional_data.as_ref().unwrap_or(&"".to_string())
        );
    }
}

fn sum_partial_intersection(zones: &Vec<Zone>, zone: &Zone, ignores: &Option<Vec<String>>) -> u64 {
    return partial_intersect(zones, zone.idx)
        .iter()
        .flat_map(|z_idx| zones.get(*z_idx))
        .filter(|&partial_z| {
            if let Some(names) = &ignores {
                return names.contains(&partial_z.name);
            }
            return true;
        })
        .map(|partial_z| zone.get_duration_intersection(partial_z))
        .sum::<u64>();
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
            let contains = filter_by_name_on_idx(
                zones,
                &contains_intersect(zones, z.idx),
                &config.ignores
            );

            let contains = filter_out_contains(
                zones,
                &partials,
                &contains,
            );

            // TODO: filter out sub contains within contains

            let partials = sum_zone_indices(zones, &z, &partials);
            let contains = sum_zone_indices(zones, &z, &contains);

            return QueryResult {
                query: "SelfTime".to_string(),
                name: z.name.clone(),
                count: z
                    .duration
                    .saturating_sub(partials)
                    .saturating_sub(contains),
                additional_data: None,
            };
        })
        .collect::<Vec<QueryResult>>();
}

pub fn run_query(
    query: &Query,
    config: &QueryConfig,
    zones: &Vec<Zone>,
) -> Result<(), TimelineError> {
    let results = match query {
        Query::SelfTime(s) => self_time_query(&s, config, zones),
    };

    for result in results {
        println!("{}", result);
    }

    return Ok(());
}

#[cfg(test)]
mod test {
    use crate::zone_search::set_zone_idx;
    use super::*;

    #[test]
    fn test_self_time_query() {
        let mut zones = vec![
            Zone::new("foo".to_string(), 8, 20),
            Zone::new("foo2".to_string(), 10, 50),
            Zone::new("foo".to_string(), 30, 40),
            Zone::new("foo".to_string(), 48, 55),
        ];

        set_zone_idx(&mut zones);

        let self_time = SelfTime {
            partial_ignore: vec!["foo".to_string()],
            node: "foo2".to_string(),
        };

        let config = QueryConfig {
            ignores: Rc::new(vec![]),
            queries: vec![],
        };

        let res = self_time_query(&self_time, &config, &zones);

        assert_eq!(res.len(), 1);
        assert_eq!(
            *res.get(0).unwrap(),
            QueryResult {
                query: "SelfTime".to_string(),
                name: "foo2".to_string(),
                count: 28,
                additional_data: None,
            }
        )
    }

    #[test]
    fn test_self_time_query_with_ignores() {
        let mut zones = vec![
            Zone::new("foo".to_string(), 8, 20),
            Zone::new("foo2".to_string(), 10, 50),
            Zone::new("ignore-me".to_string(), 25, 30),
            Zone::new("foo".to_string(), 30, 40),
            Zone::new("foo".to_string(), 48, 55),
        ];

        set_zone_idx(&mut zones);

        let self_time = SelfTime {
            partial_ignore: vec!["foo".to_string()],
            node: "foo2".to_string(),
        };

        let config = QueryConfig {
            ignores: Rc::new(vec!["ignore-me".to_string()]),
            queries: vec![],
        };

        let res = self_time_query(&self_time, &config, &zones);

        assert_eq!(res.len(), 1);
        assert_eq!(
            *res.get(0).unwrap(),
            QueryResult {
                query: "SelfTime".to_string(),
                name: "foo2".to_string(),
                count: 23,
                additional_data: None,
            }
        )
    }
}
