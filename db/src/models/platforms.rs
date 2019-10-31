// Definition in enums.rs

use itertools::Itertools;
use models::Platforms;
use prelude::ParseError;
use regex::RegexSet;
use utils::errors::DatabaseError;

impl Platforms {
    pub fn from_user_agent(agent: &str) -> Result<Platforms, DatabaseError> {
        let set = RegexSet::new(&[r"okhttp", r"Big.*Neon", r"Mozilla"])?;

        let matches = set.matches(agent).into_iter().collect_vec();

        if matches.contains(&0) || matches.contains(&1) {
            return Ok(Platforms::App);
        }
        if matches.contains(&2) {
            return Ok(Platforms::Web);
        }

        Err(ParseError {
            message: "Could not determine platform from user agent".to_string(),
            input: agent.to_string(),
        }
        .into())
    }
}
