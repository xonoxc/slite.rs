use crate::input_buffer::InputBuffer;
use std::str::FromStr;

#[derive(Debug)]
pub enum MetaCmdRes {
    MetaRecognizedCommand,
    ExitCmd,
    UnrecognizedCommand,
}

impl FromStr for MetaCmdRes {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("_") {
            return Err(());
        }

        if s == "_exit" {
            return Ok(MetaCmdRes::ExitCmd);
        }

        Ok(MetaCmdRes::MetaRecognizedCommand)
    }
}

#[derive(Debug)]
pub enum StatementType {
    StatementInsert { args: Vec<String> },
    StatementSelect,
}

impl FromStr for StatementType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.trim().split_whitespace();

        let command = parts.next().ok_or(())?;
        match command {
            ".insert" => {
                let args: Vec<String> = parts.map(|s| s.to_string()).collect();

                if args.is_empty() {
                    return Err(());
                }

                Ok(StatementType::StatementInsert { args })
            }
            ".select" => Ok(StatementType::StatementSelect),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub enum PrepareResult {
    PrepareSuccess { statement_type: StatementType },
    PrepareUnrecognizedStatement,
}

#[derive(Debug)]
pub struct Statement {
    pub statement_type: StatementType,
}

pub fn parse_statement(buffer: &InputBuffer) -> PrepareResult {
    match buffer.buffer.trim().parse::<StatementType>() {
        Ok(statement_type) => PrepareResult::PrepareSuccess { statement_type },
        Err(_) => PrepareResult::PrepareUnrecognizedStatement,
    }
}

pub fn exec_statement(statement_type: StatementType) {
    match statement_type {
        StatementType::StatementInsert { args } => {
            println!("exectuing insert! with args {:?}", args)
        }
        StatementType::StatementSelect => println!("executing select"),
    }
}
