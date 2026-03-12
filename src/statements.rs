use crate::{
    data::{
        row::Row,
        table::{ROWS_PER_PAGE, TABLE_MAX_ROWS, Table},
    },
    input_buffer::InputBuffer,
};
use std::{i32, str::FromStr};

#[derive(Debug, PartialEq)]
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

pub fn exec_statement(statement_type: StatementType, table: &mut Table) -> ExecStatementRes {
    let excution_res = match statement_type {
        StatementType::StatementInsert { ref row } => exec_insert(&row, table),
        StatementType::StatementSelect => exec_select(table),
    };

    excution_res
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ExecStatementRes {
    ExecSuccess,
    ExecFailure { cause: String },
    ExecExit,
}

fn exec_insert(row: &Row, table: &mut Table) -> ExecStatementRes {
    if table.rows >= TABLE_MAX_ROWS {
        return ExecStatementRes::ExecFailure {
            cause: "table full".to_string(),
        };
    }

    let slot = match table.get_row_slot(table.rows) {
        Ok(v) => v,
        Err(e) => {
            return ExecStatementRes::ExecFailure {
                cause: e.to_string(),
            };
        }
    };

    row.serialize(slot);
    table.rows += 1;

    ExecStatementRes::ExecSuccess
}

fn exec_select(table: &mut Table) -> ExecStatementRes {
    let mut row = Row::new();

    for i in 0..table.rows {
        let slot = match table.get_row_slot(i) {
            Ok(v) => v,
            Err(e) => {
                return ExecStatementRes::ExecFailure {
                    cause: e.to_string(),
                };
            }
        };
        row.ingest_deserialized(slot.as_ref());

        println!("{:?}", row)
    }

    ExecStatementRes::ExecSuccess
}

/*
*
* TESTS
* ***/
#[cfg(test)]
mod tests {
    use super::*;
    use crate::input_buffer::InputBuffer;

    #[test]
    fn test_parse_insert_valid() {
        let buffer = InputBuffer::from(".insert 1 testuser test@example.com".to_string());
        let result = parse_statement(&buffer);

        match result {
            PrepareResult::PrepareSuccess { statement_type } => {
                assert!(matches!(
                    statement_type,
                    StatementType::StatementInsert { row }
                    if row.id == 1 && row.username == "testuser" && row.email == "test@example.com"
                ));
            }
            _ => panic!("Expected PrepareSuccess"),
        }
    }

    #[test]
    fn test_parse_select() {
        let buffer = InputBuffer::from(".select".to_string());
        let result = parse_statement(&buffer);

        match result {
            PrepareResult::PrepareSuccess { statement_type } => {
                assert!(matches!(statement_type, StatementType::StatementSelect));
            }
            _ => panic!("Expected PrepareSuccess"),
        }
    }

    #[test]
    fn test_parse_invalid_statement() {
        let buffer = InputBuffer::from("invalid command".to_string());
        let result = parse_statement(&buffer);

        assert!(matches!(
            result,
            PrepareResult::PrepareUnrecognizedStatement
        ));
    }

    #[test]
    fn test_parse_insert_invalid_id() {
        let buffer = InputBuffer::from(".insert abc testuser test@example.com".to_string());
        let result = parse_statement(&buffer);

        assert!(matches!(
            result,
            PrepareResult::PrepareUnrecognizedStatement
        ));
    }

    #[test]
    fn test_parse_insert_missing_args() {
        let buffer = InputBuffer::from(".insert 1 testuser".to_string());
        let result = parse_statement(&buffer);

        assert!(matches!(
            result,
            PrepareResult::PrepareUnrecognizedStatement
        ));
    }

    #[test]
    fn test_meta_command_exit() {
        let result = "_exit".parse::<MetaCmdRes>();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), MetaCmdRes::ExitCmd);
    }

    #[test]
    fn test_meta_command_unrecognized() {
        let result = "random".parse::<MetaCmdRes>();
        assert!(result.is_err());
    }
}
