use crate::{zone_search::{filter_by_names_on_idx, get_partial_contained, get_contained, filter_out_contains, sum_zone_indices, filter_by_name_on_idx}, zones::Zone};

pub fn calculate_self_time(zone_idx: usize, zones: &Vec<Zone>, partial_ignores: &Vec<String>, ignores: &Vec<String>) -> u64 {
    let zone = zones.get(zone_idx).unwrap();
    let partials = filter_by_names_on_idx(
        zones,
        &get_partial_contained(zones, zone.idx),
        &partial_ignores,
    );

    let contains =
        filter_by_names_on_idx(zones, &get_contained(zones, zone.idx), &ignores);

    let contains = filter_out_contains(zones, &partials, &contains);

    // TODO: filter out sub contains within contains

    let partials = sum_zone_indices(zones, &zone, &partials);
    let contains = sum_zone_indices(zones, &zone, &contains);

    return zone.duration.saturating_sub(partials).saturating_sub(contains);
}

pub fn get_start_of_cpp(zones: &Vec<Zone>, parents: &Vec<usize>) -> Option<usize> {
    let mut start_of_cpp_call: Option<usize> = None;
    {
        let possible = filter_by_name_on_idx(zones, &parents, "V8.Builtin_HandleApiCall");
        if possible.is_empty() {
            let possible = filter_by_name_on_idx(zones, &parents, "V8.ExternalCallback");
            if !possible.is_empty() {
                start_of_cpp_call = Some(*possible.get(0).unwrap());
            }
        } else {
            start_of_cpp_call = Some(*possible.get(0).unwrap());
        }
    }

    return start_of_cpp_call;
}

pub fn calculate_total_time(zone: &Zone, zones: &Vec<Zone>, ignores: &Vec<String>) -> u64 {
    let contains =
        filter_by_names_on_idx(zones, &get_contained(zones, zone.idx), &ignores);

    // TODO: filter out sub contains within contains

    let contains = sum_zone_indices(zones, &zone, &contains);

    return zone.duration.saturating_sub(contains);
}

pub fn get_impl_arg(zones: &Vec<Zone>, parent: usize) -> Option<usize> {
    // TODO: toImplArgs?  There are two versions of these, but it doesn't seem to be used very
    // much.
    let contains =
        filter_by_name_on_idx(zones, &get_contained(zones, parent), "toImplArgs2");

    if contains.is_empty() {
        return None;
    }

    return Some(*contains.get(0).unwrap());
}
