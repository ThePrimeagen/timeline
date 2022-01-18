use std::str::FromStr;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct QueryConfig {
    pub ignores: Vec<String>,
}

impl FromStr for QueryConfig {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        return Ok(serde_json::from_str::<QueryConfig>(s)?);
    }
}

