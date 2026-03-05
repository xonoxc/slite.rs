use crate::{
    data::{
        row::Row,
        table::{TABLE_MAX_ROWS, Table},
    },
    input_buffer::InputBuffer,
};
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

pub fn exec_statement(statement_type: StatementType, table: &mut Table) {
    let excution_res = match statement_type {
        StatementType::StatementInsert { ref row } => exec_insert(&row, table),
        StatementType::StatementSelect => exec_select(table),
    };

    match excution_res {
        ExecStatementRes::ExecFailure { cause } => println!("Excution failed :: {}", cause),
        ExecStatementRes::ExecSuccess => println!("Operation success!!"),
    };
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ExecStatementRes {
    ExecSuccess,
    ExecFailure { cause: &'static str },
}

fn exec_insert(row: &Row, table: &mut Table) -> ExecStatementRes {
    if table.rows >= TABLE_MAX_ROWS {
        return ExecStatementRes::ExecFailure {
            cause: "table full",
        };
    }

    let slot = table.get_row_slot(table.rows);
    row.serialize(slot);
    table.rows += 1;

    ExecStatementRes::ExecSuccess
}

fn exec_select(table: &mut Table) -> ExecStatementRes {
    let mut row = Row::new();

    for i in 0..table.rows {
        let slot = table.get_row_slot(i);
        row.ingest_deserialized(slot.as_ref());
        println!("{:?}", row)
    }

    ExecStatementRes::ExecSuccess
}
