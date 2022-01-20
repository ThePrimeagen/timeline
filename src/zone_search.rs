use log::{info, debug};

use crate::zones::Zone;

pub struct ZoneIdx(usize);

impl Into<ZoneIdx> for usize {
    fn into(self) -> ZoneIdx {
        return ZoneIdx(self);
    }
}

pub fn set_zone_idx(vec: &mut Vec<Zone>) {
    vec.iter_mut().enumerate().for_each(|(idx, z)| {
        z.idx = idx;
    });
}

// TODO: Hashmap??
pub fn get_by_name(vec: &Vec<Zone>, name: &str) -> Vec<usize> {
    let mut out = vec![];

    for zone in vec {
        if zone.name == name {
            out.push(zone.idx);
        }
    }

    return out;
}

pub enum FilterResult {
    Continue,
    Add,
    Break,
}

fn partial_intersect_filter(zone: &Zone, z: &Zone) -> FilterResult {
    if zone.partial_contains(z) {
        return FilterResult::Add;
    } else if zone.contains(z) || z.contains(zone) {
        return FilterResult::Continue;
    }
    return FilterResult::Break;
}

fn contain_intersect_filter(zone: &Zone, z: &Zone) -> FilterResult {
    if zone.contains(z) {
        return FilterResult::Add;
    }
    return FilterResult::Break;
}

fn intersect_by_filter(
    zones: &Vec<Zone>,
    idx: usize,
    filter: Box<dyn Fn(&Zone, &Zone) -> FilterResult>,
) -> Vec<usize> {
    let mut out = vec![];
    let zone = zones
        .get(idx)
        .expect("should never hand a zone that doesn't exist");
    let mut curr_idx = idx;

    loop {
        let next_idx = curr_idx.saturating_sub(1);
        if next_idx == curr_idx {
            break;
        }

        if let Some(z) = zones.get(next_idx) {
            match filter(zone, z) {
                FilterResult::Add => {
                    out.push(z.idx);
                }
                FilterResult::Break => {
                    break;
                }
                _ => {}
            }
        }

        curr_idx = next_idx;
    }

    let mut curr_idx = idx;
    loop {
        let next_idx = curr_idx + 1;
        if next_idx >= zones.len() {
            break;
        }

        if let Some(z) = zones.get(next_idx) {
            match filter(zone, z) {
                FilterResult::Add => {
                    out.push(z.idx);
                }
                FilterResult::Break => {
                    break;
                }
                _ => {}
            }
        }

        curr_idx = next_idx;
    }

    return out;
}

pub fn sum_zone_indices(zones: &Vec<Zone>, zone: &Zone, containers: &Vec<usize>) -> u64 {
    return containers
        .iter()
        .map(|z_idx| {
            let other_zone = zones
                .get(*z_idx)
                .expect("all indices should be valid");
            return zone.get_duration_intersection(other_zone);
        })
        .sum();
}

// TODO: I could get really clever with this algo and make it o(N), but
// that is hard and I don't want to do it...
pub fn filter_out_contains(
    zones: &Vec<Zone>,
    containers: &Vec<usize>,
    possible_contains: &Vec<usize>,
) -> Vec<usize> {
    let mut out = vec![];

    for possible in possible_contains {
        let possible_zone = zones.get(*possible).expect("all indices should be valid");
        let mut contained = false;

        for container in containers {
            contained = zones
                .get(*container)
                .expect("all indices should be valid")
                .contains(&possible_zone);
            if contained {
                break;
            }
        }

        if !contained {
            out.push(*possible);
        }
    }

    return out;
}

pub fn filter_by_name_on_idx(
    zones: &Vec<Zone>,
    filter_zones: &Vec<usize>,
    names: &Vec<String>,
) -> Vec<usize> {
    let mut out = vec![];

    for zone_idx in filter_zones {
        let zone = zones.get(*zone_idx).expect("all indices should be valid");
        if names.contains(&zone.name) {
            out.push(zone.idx);
        }
    }

    return out;
}

pub fn filter_by_name(zones: &Vec<Zone>, names: &Vec<String>) -> Vec<usize> {
    let mut out = vec![];

    for zone in zones {
        debug!("filter_by_name: {:?} -- {:?}", names, zone.name);
        if names.contains(&zone.name) {
            out.push(zone.idx);
        }
    }

    return out;
}

pub fn contains_intersect(zones: &Vec<Zone>, idx: usize) -> Vec<usize> {
    return intersect_by_filter(zones, idx, Box::new(contain_intersect_filter));
}

pub fn partial_intersect(zones: &Vec<Zone>, idx: usize) -> Vec<usize> {
    return intersect_by_filter(zones, idx, Box::new(partial_intersect_filter));
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use crate::tests::TestZone;

    #[test]
    fn test_filter_by_name() {
        let mut zones = vec![
            Zone::new("foo".to_string(), 8, 20, 0),
            Zone::new("foo2".to_string(), 10, 50, 0),
            Zone::new("foo".to_string(), 30, 40, 0),
            Zone::new("foo4".to_string(), 32, 56, 0),
            Zone::new("foo".to_string(), 48, 55, 0),
            Zone::new("foo6".to_string(), 55, 65, 0),
        ];
        set_zone_idx(&mut zones);

        let filtered_zones = filter_by_name(&zones, &vec!["foo".to_string()]);
        assert_eq!(filtered_zones.len(), 3);
        assert_eq!(filtered_zones.get(0).unwrap(), &0);
        assert_eq!(filtered_zones.get(1).unwrap(), &2);
        assert_eq!(filtered_zones.get(2).unwrap(), &4);
    }

    #[test]
    fn test_get_by_name() {
        let mut zones = vec![
            Zone::new("foo".to_string(), 8, 20, 0),
            Zone::new("foo2".to_string(), 10, 50, 0),
            Zone::new("foo".to_string(), 30, 40, 0),
            Zone::new("foo4".to_string(), 32, 56, 0),
            Zone::new("foo".to_string(), 48, 55, 0),
            Zone::new("foo6".to_string(), 55, 65, 0),
        ];
        set_zone_idx(&mut zones);

        assert_eq!(get_by_name(&zones, "foo"), vec![0, 2, 4],);
    }
    #[test]
    fn test_partial_intersection() {
        let mut zones = vec![
            Zone::from_timestamps(8, 20),
            Zone::from_timestamps(10, 50),
            Zone::from_timestamps(30, 40),
            Zone::from_timestamps(32, 56),
            Zone::from_timestamps(48, 55),
            Zone::from_timestamps(55, 65),
        ];
        set_zone_idx(&mut zones);

        assert_eq!(partial_intersect(&zones, 3), vec![2, 1, 5]);

        assert_eq!(partial_intersect(&zones, 5), vec![4, 3]);

        assert_eq!(partial_intersect(&zones, 0), vec![1]);
    }

    #[test]
    fn test_partial_intersection_with_super_container() {
        let mut zones = vec![
            Zone::from_timestamps(8, 55),
            Zone::from_timestamps(10, 50),
        ];
        set_zone_idx(&mut zones);

        let expected: Vec<usize> = vec![];
        assert_eq!(partial_intersect(&zones, 1), expected);
    }

    #[test]
    fn test_filter_out_contains() {
        let mut zones = vec![
            Zone::from_timestamps(8, 20),
            Zone::from_timestamps(15, 18),
            Zone::from_timestamps(10, 50),
            Zone::from_timestamps(40, 42),
        ];
        set_zone_idx(&mut zones);

        let partials = vec![0];

        let contains = vec![1];
        let contains = filter_out_contains(&zones, &partials, &contains);
        assert_eq!(contains.len(), 0);

        let contains = vec![1, 3];
        let contains = filter_out_contains(&zones, &partials, &contains);
        assert_eq!(contains.len(), 1);
        assert_eq!(contains.get(0).unwrap(), &3);
    }
}
