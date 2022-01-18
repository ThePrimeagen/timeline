use std::rc::Rc;

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
    } else if zone.contains(z) {
        return FilterResult::Continue;
    }
    return FilterResult::Break;
}

fn intersect_by_filter(zones: &Vec<Zone>, idx: usize, filter: Box<dyn Fn(&Zone, &Zone) -> FilterResult>) -> Vec<usize> {
    let mut out = vec![];
    let zone = zones.get(idx).expect("should never hand a zone that doesn't exist");
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
                },
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
                },
                _ => {}
            }
        }

        curr_idx = next_idx;
    }

    return out;
}

pub fn contained_ignores(zones: &Vec<Zone>, idx: usize, ignores: Rc<Vec<String>>) -> Vec<usize> {

    return intersect_by_filter(zones, idx, Box::new(move |starting_zone, zone| {
        if starting_zone.completes_before(zone) ||
           zone.completes_before(starting_zone) {
           return FilterResult::Break;
        }

        if starting_zone.contains(zone) {
            if ignores.contains(&zone.name) {
                return FilterResult::Add;
            }
        }

        return FilterResult::Continue;
    }));
}

pub fn partial_intersect(zones: &Vec<Zone>, idx: usize) -> Vec<usize> {
    return intersect_by_filter(zones, idx, Box::new(partial_intersect_filter));
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_by_name() {
        let mut zones = vec![
            Zone::new("foo".to_string(), 8, 20),
            Zone::new("foo2".to_string(), 10, 50),
            Zone::new("foo".to_string(), 30, 40),
            Zone::new("foo4".to_string(), 32, 56),
            Zone::new("foo".to_string(), 48, 55),
            Zone::new("foo6".to_string(), 55, 65),
        ];
        set_zone_idx(&mut zones);

        assert_eq!(
            get_by_name(&zones, "foo"),
            vec![0, 2, 4],
        );
    }
    #[test]
    fn test_partial_intersection() {
        let mut zones = vec![
            Zone::new("foo".to_string(), 8, 20),
            Zone::new("foo".to_string(), 10, 50),
            Zone::new("foo".to_string(), 30, 40),
            Zone::new("foo".to_string(), 32, 56),
            Zone::new("foo".to_string(), 48, 55),
            Zone::new("foo".to_string(), 55, 65),
        ];
        set_zone_idx(&mut zones);

        assert_eq!(
            partial_intersect(&zones, 3),
            vec![2, 1, 5]
        );

        assert_eq!(
            partial_intersect(&zones, 5),
            vec![4, 3]
        );

        assert_eq!(
            partial_intersect(&zones, 0),
            vec![1]
        );
    }
}

