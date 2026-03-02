use std::str::FromStr;

use crate::statements::{MetaCmdRes, StatementType};

#[derive(Debug)]
pub enum CLIcommand {
    Meta(MetaCmdRes),
    Statement(StatementType),
}

impl FromStr for CLIcommand {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(meta) = s.parse::<MetaCmdRes>() {
            return Ok(CLIcommand::Meta(meta));
        }

        if let Ok(statement_type) = s.parse::<StatementType>() {
            return Ok(CLIcommand::Statement(statement_type));
        }

        Err(())
    }
}
