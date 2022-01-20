use std::{str::FromStr, fs::File, io::BufReader};

use serde::Deserialize;

use super::query::Query;

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


