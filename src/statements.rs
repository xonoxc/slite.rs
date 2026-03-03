use crate::{input_buffer::InputBuffer, row::Row};
use std::{i32, str::FromStr};

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
    StatementInsert { row: Row },
    StatementSelect,
}

impl FromStr for StatementType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.trim().split_whitespace();

        let command = parts.next().ok_or(())?;
        match command {
            ".insert" => {
                let args: Vec<&str> = parts.collect();

                match args.as_slice() {
                    [id_str, username, email] => {
                        let id = id_str.parse::<i32>().map_err(|_| ())?;

                        Ok(StatementType::StatementInsert {
                            row: Row {
                                id,
                                username: username.to_string(),
                                email: email.to_string(),
                            },
                        })
                    }
                    _ => Err(()),
                }
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
        StatementType::StatementInsert { row } => println!("exectuing insert! with args {:?}", row),
        StatementType::StatementSelect => println!("executing select"),
    }
}
