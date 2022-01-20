pub mod calculations;
pub mod query;
pub mod query_config;

use log::{info, debug};

use crate::{
    error::TimelineError,
    zone_search::{
        filter_by_name, filter_by_names, get_by_name, get_contained, get_parents, get_partial_contained
    },
    zones::Zone,
};

use self::{
    calculations::{calculate_self_time, get_start_of_cpp, calculate_total_time, get_impl_arg},
    query::{Cost, DataPoint, Query, QueryResult, Reduce, SelfTime, Stat, StatResult, CostResult},
    query_config::QueryConfig,
};

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
            return QueryResult::DataPoint(DataPoint {
                query: "SelfTime".to_string(),
                name: z.name.clone(),
                count: calculate_self_time(z.idx, zones, &query.partial_ignore, &config.ignores),
                additional_data: None,
            });
        })
        .collect::<Vec<QueryResult>>();
}

pub fn reduce_query(query: &Reduce, zones: &Vec<Zone>) -> Vec<QueryResult> {
    let names = vec![query.node.clone()];
    info!("reduce_query#filte_by_name: {:?}", names);

    let found_idxs = filter_by_names(zones, &names);
    info!("found indices: {:?}", found_idxs.len());

    let found = *found_idxs
        .iter()
        .nth(query.ignore_count.unwrap_or(0))
        .expect("to always find a reduce result");

    let found = zones.get(found).expect("all indices should be valid");

    let partials = get_partial_contained(zones, found.idx);
    let contains = get_contained(zones, found.idx);

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
    return filter_by_names(zones, &vec![stat.node.clone()])
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
    let mut out = vec![];
    let tmp_vec = &vec![];

    for zone_idx in filter_by_name(zones, &cost.node) {
        let zone = zones.get(zone_idx).unwrap();
        let self_time = calculate_self_time(zone.idx, zones, &tmp_vec, &config.ignores);
        let parents = get_parents(zones, zone_idx);
        let start_of_cpp = get_start_of_cpp(zones, &parents);

        // this isn't a proper measurement
        if start_of_cpp.is_none() {
            debug!("dropping ({}): {} from cost_query due to missing start of cpp", zone.idx, &zone.name);
            continue;
        }
        let start_of_cpp = start_of_cpp.unwrap();
        let start_of_cpp = zones.get(start_of_cpp).unwrap();

        let to_cpp_total_time = calculate_total_time(start_of_cpp, zones, &config.ignores);

        let impl_arg = get_impl_arg(zones, start_of_cpp.idx);
        let impl_time = if let Some(arg) = impl_arg {
            calculate_self_time(arg, zones, &tmp_vec, &config.ignores)
        } else {
            0
        };

        out.push(QueryResult::Cost(CostResult {
            name: zone.name.clone(),
            cost_of_args: impl_time,
            cost_of_javascript: to_cpp_total_time - self_time,
            cpp_duration: self_time,
        }));
    }

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
    use crate::{zone_search::set_zone_idx, tests::TestZone};

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

    #[test]
    fn test_cost_query() -> Result<(), std::num::ParseIntError> {
        // taken from actual data from the odroid. for a single V8.Builtin_HandleApiCall
        let mut zones = Zone::from_csv_strings(vec![
            "TM_ZONE,65536,V8TracingController.AddTraceEvent,1642630333023844044,1642630333023848502,10,0x0,0xff0000ff,include/nrdbase/Telemetry.h,86,0,0",
            "TM_ZONE,65536,V8.ExternalCallback,1642630333023852127,1642630333023877210,7,0x0,0x0,/home/mpaulson/.pvm/installed/odroid-x86_64/32-release-21.2/dev/.pvm/build/src/src/base/Telemetry.h,111,0,0",
            "TM_ZONE,65536,INST_DataBufferBridge_CLASSgetUint8,1642630333023854211,1642630333023867793,8,0x0,0x0,/home/mpaulson/.pvm/installed/odroid-x86_64/32-release-21.2/dev/.pvm/build/src/src/base/Telemetry.h,111,0,0",
            "TM_ZONE,65536,toImplArgs2,1642630333023856211,1642630333023858544,9,0x0,0x0,/home/mpaulson/.pvm/installed/odroid-x86_64/32-release-21.2/dev/.pvm/build/src/src/base/Telemetry.h,111,0,0",
            "TM_ZONE,65536,DataBufferBridge.getUint8,1642630333023863169,1642630333023865376,9,0x0,0x0,/home/mpaulson/.pvm/installed/odroid-x86_64/32-release-21.2/dev/.pvm/build/src/src/base/Telemetry.h,111,0,0",
            "TM_ZONE,65536,V8TracingController.AddTraceEvent,1642630333023869126,1642630333023873710,10,0x0,0xff0000ff,include/nrdbase/Telemetry.h,86,0,0",
            "TM_ZONE,65536,V8.Builtin_HandleApiCall,1642630333023839586,1642630333023880460,6,0x0,0x0,/home/mpaulson/.pvm/installed/odroid-x86_64/32-release-21.2/dev/.pvm/build/src/src/base/Telemetry.h,111,0,0",
        ])?;
        set_zone_idx(&mut zones);

        let result = cost_query(&Cost {
            node: "DataBufferBridge.getUint8".to_string(),
        }, &QueryConfig {
            ignores: vec!["V8TracingController.AddTraceEvent".to_string()],
            queries: vec![],
        }, &zones);

        assert_eq!(result.len(), 1);

        match result.get(0).unwrap() {
            QueryResult::Cost(c) => {
                let cpp_duration = 1642630333023865376u64 - 1642630333023863169u64;
                let args = 1642630333023858544u64 - 1642630333023856211u64;
                let tracing =
                    1642630333023848502u64 - 1642630333023844044u64 +
                    1642630333023873710u64 - 1642630333023869126u64;
                let to_cpp_duration = 1642630333023880460 - 1642630333023839586u64;

                assert_eq!(c.cpp_duration, cpp_duration);
                assert_eq!(c.cost_of_args, args);
                assert_eq!(c.cost_of_javascript, to_cpp_duration - cpp_duration - tracing);
            }
            _ => unreachable!(),
        }

        return Ok(());
    }
}
