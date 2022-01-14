use std::{collections::HashSet, hash::Hash};

use log::debug;

use crate::{query::QueryConfig, zone::Zone};

// cuts off time display to this length (% TIME_CUT_OFF)
const TIME_CUT_OFF: u64 = 10_000_000;

/// Node is for building the timeline querying only.
#[derive(Debug, Eq)]
pub struct Node {
    pub len: usize,
    pub zone: Zone,
    pub children: Vec<Node>,
}

impl Hash for Node {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.len.hash(state);
        self.zone.hash(state);
        self.children.hash(state);
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        return self.zone == other.zone;
    }
}

pub enum TimeCalculation {
    Start,
    End,
}

impl Node {
    pub fn new(zone: Zone) -> Node {
        return Node {
            len: 0,
            zone,
            children: vec![],
        };
    }

    pub fn push(&mut self, node: Node) {
        self.len += 1;

        let mut container: Option<usize> = None;
        for (idx, child) in self.children.iter().enumerate() {
            if child.zone.contains(&node.zone) {
                container = Some(idx);
                break;
            }
        }

        if let Some(idx) = container {
            self.children.get_mut(idx).unwrap().push(node);
        } else {
            self.children.push(node);
        }
    }

    pub fn print(&self, depth: usize) {
        for _ in 0..(depth * 4) {
            print!(" ");
        }
        println!("{}", self.to_string());

        self.children.iter().for_each(|c| c.print(depth + 1));
    }

    pub fn contains_zone(&self, name: &str) -> bool {
        if self.zone.name == name {
            return true;
        }
        for child in &self.children {
            if child.contains_zone(name) {
                return true;
            }
        }

        return false;
    }

    pub fn child_by_name(&self, name: &str) -> Option<&Node> {
        if self.zone.name == name {
            return Some(&self);
        }

        for child in &self.children {
            if child.contains_zone(name) {
                return child.child_by_name(name);
            }
        }

        return None;
    }

    pub fn nodes_by_name(&self, name: &str) -> Vec<&Node> {
        debug!("Node#nodes_by_name({}): {}", self.zone.name, name);
        if self.zone.name == name {
            return vec![self];
        }

        return self
            .children
            .iter()
            .flat_map(|n| n.nodes_by_name(name))
            .collect::<Vec<&Node>>();
    }

    /// +----------------------------------------------+
    /// |                A                             |
    /// s----------------------------------------------e
    ///  +----------+     +----------+   +----------+
    ///  | B (...)  |     |    C     |   | D (...)  |
    ///  s----------e     s----------e   s----------e
    ///
    ///  time_to = (As - Cs) - Bdur
    pub fn time_to(&self, name: &str, ignores: &[String], side: &TimeCalculation) -> Option<u64> {
        if !self.contains_zone(name) {
            return None;
        }

        let child = match self.child_by_name(name) {
            Some(c) => c,
            _ => return None,
        };

        let mut ignore_duration = ignores
            .iter()
            .flat_map(|i| self.nodes_by_name(i))
            .filter(|n| {
                return if let TimeCalculation::Start = side {
                    n.zone.completes_by(&child.zone)
                } else {
                    child.zone.completes_by(&n.zone)
                };
            })
            .collect::<HashSet<&Node>>();

        let distance: u64;
        if let TimeCalculation::Start = side {
            // B from the above diagram
            self.children
                .iter()
                .filter(|c| c.zone.completes_by(&child.zone))
                .for_each(|c| {
                    if !ignore_duration.contains(c) {
                        ignore_duration.insert(c);
                    }
                });

            distance = self.zone.get_start_distance(&child.zone);
        } else {
            // D from the above diagram
            self.children
                .iter()
                .filter(|c| child.zone.completes_by(&c.zone))
                .for_each(|c| {
                    if !ignore_duration.contains(c) {
                        ignore_duration.insert(c);
                    }
                });
            distance = self.zone.get_end_distance(&child.zone);
        }

        let ignore_duration = ignore_duration
            .iter()
            .fold(0, |acc, n| acc + n.calc_self_time());
        debug!(
            "Node#time_to({} - {}): {} - {} = {}",
            self.zone.name,
            child.zone.name,
            distance,
            ignore_duration,
            distance - ignore_duration
        );
        return Some(distance - ignore_duration);
    }

    pub fn calc_child_self_time(&self, name: &str) -> Option<u64> {
        return match self.child_by_name(name) {
            Some(n) => Some(n.calc_self_time()),
            _ => None,
        };
    }

    pub fn calc_self_time(&self) -> u64 {
        let mut dur = self.zone.duration;
        for child in &self.children {
            dur -= child.zone.duration;
        }

        return dur;
    }

    pub fn calc_total_time(&self, ignores: &[String]) -> u64 {
        let dur = self.zone.duration;
        let ignore_duration = ignores
            .iter()
            .flat_map(|i| self.nodes_by_name(i))
            .map(|n| n.zone.duration)
            .sum::<u64>();

        return dur - ignore_duration;
    }
}

impl ToString for Node {
    fn to_string(&self) -> String {
        return format!(
            "{}({}) ({} - {})({})",
            self.zone.name,
            self.children.len(),
            self.zone.time_start % TIME_CUT_OFF,
            self.zone.time_end % TIME_CUT_OFF,
            self.zone.duration
        );
    }
}

impl From<Zone> for Node {
    fn from(z: Zone) -> Self {
        return Node::new(z);
    }
}

pub fn build_trees(zones: Vec<Zone>, query: &QueryConfig) -> Vec<Node> {
    let (roots, nodes): (Vec<Node>, Vec<Node>) = zones
        .into_iter()
        .map(Node::from)
        .partition(|node| node.zone.name == query.root);

    let mut nodes = nodes.into_iter();
    let mut roots = roots.into_iter();
    let mut root = roots.next().unwrap();
    let mut node = nodes.next().unwrap();
    let mut next_roots: Vec<Node> = vec![];

    loop {
        debug!(
            "build_trees node({}) root({})",
            node.to_string(),
            root.to_string()
        );
        if root.zone.contains(&node.zone) {
            root.push(node);
            if let Some(n) = nodes.next() {
                node = n;
            } else {
                break;
            }
        } else if root.zone.ends_after(&node.zone) {
            if let Some(n) = nodes.next() {
                node = n;
            } else {
                break;
            }
        } else {
            if let Some(l) = roots.next() {
                next_roots.push(root);
                root = l;
            } else {
                break;
            }
        }
    }
    next_roots.push(root);

    return next_roots;
}

#[cfg(test)]
/// Note that on tests I get lazy and simply just do box dyn errors because
/// nothing should error here.
mod test {
    use std::collections::HashSet;

    use super::*;
    use crate::{query::QueryConfig, zone::Zone};

    #[test]
    fn test_build_trees() {
        let zones = vec![
            Zone::new("A".to_string(), 50, 100),
            Zone::new("B".to_string(), 68, 71),
            Zone::new("C".to_string(), 68, 70),
        ];

        let query: QueryConfig = QueryConfig {
            root: "A".to_string(),
            zones: HashSet::new(),
            queries: vec![],
            ignores: vec![],
        };

        let roots = build_trees(zones, &query);

        assert_eq!(roots.len(), 1);
        assert_eq!(
            roots.get(0).unwrap().child_by_name("B").unwrap().zone.name,
            "B"
        );
        assert_eq!(
            roots
                .get(0)
                .unwrap()
                .child_by_name("B")
                .unwrap()
                .child_by_name("C")
                .unwrap()
                .zone
                .name,
            "C"
        );
    }

    #[test]
    fn test_build_trees_complex() {
        let zones = vec![
            Zone::new("A".to_string(), 50, 100),
            Zone::new("D".to_string(), 48, 50),
            Zone::new("B".to_string(), 68, 71),
            Zone::new("C".to_string(), 68, 70),
            Zone::new("A".to_string(), 101, 120),
            Zone::new("B".to_string(), 110, 112),
            Zone::new("D".to_string(), 122, 125),
        ];

        let query: QueryConfig = QueryConfig {
            root: "A".to_string(),
            zones: HashSet::new(),
            queries: vec![],
            ignores: vec![],
        };

        let roots = build_trees(zones, &query);
        assert_eq!(roots.len(), 2);

        let b_children = roots.get(0).unwrap().nodes_by_name("B");
        let c_children = roots.get(0).unwrap().nodes_by_name("C");

        assert_eq!(b_children.len(), 1);
        assert_eq!(c_children.len(), 1);

        let b_children = roots.get(1).unwrap().nodes_by_name("B");
        let c_children = roots.get(1).unwrap().nodes_by_name("C");

        assert_eq!(b_children.len(), 1);
        assert_eq!(c_children.len(), 0);
    }

    #[test]
    fn test_search() -> Result<(), Box<dyn std::error::Error>> {
        let head_zone = Zone::new("foo".to_string(), 0, 100);
        let mut head = Node::new(head_zone);

        head.push(Node::new(Zone::new("bar".to_string(), 40, 50)));
        head.push(Node::new(Zone::new("buzz".to_string(), 42, 48)));
        head.push(Node::new(Zone::new("bluh".to_string(), 43, 47)));

        assert_eq!(head.contains_zone("bar"), true);
        assert_eq!(head.contains_zone("buzz"), true);
        assert_eq!(head.contains_zone("bluh"), true);
        assert_eq!(head.contains_zone("blazz"), false);

        assert_eq!(head.child_by_name("bar").unwrap().zone.name, "bar");
        assert_eq!(head.child_by_name("buzz").unwrap().zone.name, "buzz");
        assert_eq!(head.child_by_name("bluh").unwrap().zone.name, "bluh");

        assert_eq!(
            head.child_by_name("bar")
                .unwrap()
                .child_by_name("buzz")
                .unwrap()
                .zone
                .name,
            "buzz"
        );

        assert_eq!(
            head.child_by_name("bar")
                .unwrap()
                .child_by_name("buzz")
                .unwrap()
                .child_by_name("bluh")
                .unwrap()
                .zone
                .name,
            "bluh"
        );
        return Ok(());
    }

    #[test]
    fn test_calc() -> Result<(), Box<dyn std::error::Error>> {
        let head_zone = Zone::new("foo".to_string(), 0, 100);
        let mut head = Node::new(head_zone);

        head.push(Node::new(Zone::new("before-bar".to_string(), 20, 38)));
        head.push(Node::new(Zone::new("before-bar-2".to_string(), 15, 18)));
        head.push(Node::new(Zone::new("bar".to_string(), 40, 50)));
        head.push(Node::new(Zone::new("buzz".to_string(), 41, 48)));
        head.push(Node::new(Zone::new("drop-pre".to_string(), 41, 42)));
        head.push(Node::new(Zone::new("drop-post".to_string(), 46, 47)));
        head.push(Node::new(Zone::new("bluh".to_string(), 43, 44)));
        head.push(Node::new(Zone::new("after-bar".to_string(), 70, 75)));
        head.push(Node::new(Zone::new("after-bar-2".to_string(), 77, 78)));

        // self times
        assert_eq!(head.calc_self_time(), 63);
        assert_eq!(head.calc_child_self_time("buzz").unwrap(), 4);
        assert_eq!(head.calc_child_self_time("bluh").unwrap(), 1);
        assert_eq!(head.calc_child_self_time("blazz"), None);
        assert_eq!(head.calc_total_time(&vec![]), 100);
        assert_eq!(head.calc_total_time(&vec!["drop-pre".to_string()]), 100 - 1);
        assert_eq!(
            head.calc_total_time(&vec!["drop-pre".to_string(), "drop-post".to_string()]),
            100 - 1 - 1
        );

        // distances
        let b_sum = 38 - 20 + 18 - 15;
        assert_eq!(
            head.time_to("buzz", &vec![], &TimeCalculation::Start)
                .unwrap(),
            41 - 0 - b_sum
        );
        assert_eq!(
            head.time_to("bluh", &vec![], &TimeCalculation::Start)
                .unwrap(),
            43 - 0 - b_sum
        );
        assert_eq!(
            head.time_to(
                "bluh",
                &vec!["drop-pre".to_string()],
                &TimeCalculation::Start
            )
            .unwrap(),
            43 - 0 - b_sum - 1
        );

        let d_sum = 75 - 70 + 78 - 77;
        assert_eq!(
            head.time_to("buzz", &vec![], &TimeCalculation::End)
                .unwrap(),
            100 - 48 - d_sum
        );
        assert_eq!(
            head.time_to("bluh", &vec![], &TimeCalculation::End)
                .unwrap(),
            100 - 44 - d_sum
        );
        assert_eq!(
            head.time_to(
                "bluh",
                &vec!["drop-post".to_string()],
                &TimeCalculation::End
            )
            .unwrap(),
            100 - 44 - d_sum - 1
        );

        return Ok(());
    }
}
