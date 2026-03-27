use crate::{
    cursor::Cursor,
    data::{
        row::{ROW_SIZE, Row},
        table::{MAX_EMAIL_SIZE, MAX_USERNAME_SIZE, Table},
    },
    input_buffer::InputBuffer,
    trees::{
        consts::{
            COMMON_HEADER_SIZE, LEAF_NODE_CELL_SIZE, LEAF_NODE_HEADER_SIZE, LEAF_NODE_MAX_CELLS,
            LEAF_NODE_SPACE_FOR_CELLS,
        },
        page_node::Page,
    },
};
use std::{i32, str::FromStr};

use std::io::{self, Write};

#[derive(Debug, PartialEq)]
pub enum MetaCmdRes {
    MetaRecognizedCommand,
    ExitCmd,
    MetaCmdClear,
    MetaCmdBtree,
    MetaCmdConstants,
    UnrecognizedCommand,
}

impl FromStr for MetaCmdRes {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("_") {
            return Err(());
        }

        match s {
            "_exit" => return Ok(MetaCmdRes::ExitCmd),
            "_cl" => return Ok(MetaCmdRes::MetaCmdClear),
            "_btree" => return Ok(MetaCmdRes::MetaCmdBtree),
            "_constants" => return Ok(MetaCmdRes::MetaCmdConstants),
            _ => Ok(MetaCmdRes::MetaRecognizedCommand),
        }
    }
}

#[derive(Debug)]
pub enum StatementType {
    StatementInsert { row: Row },
    StatementSelect,
}

impl FromStr for StatementType {
    type Err = PrepareResult;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.trim().split_whitespace();

        let command = parts.next().ok_or(PrepareResult::PrepareSyntaxError {
            cause: "unable to parse command",
        })?;

        match command {
            ".insert" => {
                let args: Vec<&str> = parts.collect();

                match args.as_slice() {
                    [id_str, username, email] => {
                        let id = id_str
                            .parse::<i32>()
                            .map_err(|_| PrepareResult::IdParseErr)?;

                        if id < 0 {
                            return Err(PrepareResult::PrepareNegativeId);
                        }

                        if email.len() > MAX_EMAIL_SIZE {
                            return Err(PrepareResult::PrepareStringTooLong {
                                cause: "email string too long",
                            });
                        }

                        if username.len() > MAX_USERNAME_SIZE {
                            return Err(PrepareResult::PrepareStringTooLong {
                                cause: "username too long",
                            });
                        }

                        Ok(StatementType::StatementInsert {
                            row: Row {
                                id,
                                username: username.to_string(),
                                email: email.to_string(),
                            },
                        })
                    }
                    _ => Err(PrepareResult::PrepareSyntaxError {
                        cause: "expected .insert <id> <username> <email>",
                    }),
                }
            }
            ".select" => Ok(StatementType::StatementSelect),
            _ => Err(PrepareResult::PrepareUnrecognizedStatement),
        }
    }
}

#[derive(Debug)]
pub enum PrepareResult {
    PrepareSuccess { statement_type: StatementType },
    PrepareUnrecognizedStatement,
    PrepareSyntaxError { cause: &'static str },
    PrepareStringTooLong { cause: &'static str },
    PrepareNegativeId,
    IdParseErr,
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

pub fn exec_statement(statement_type: StatementType, cursor: &mut Cursor) -> ExecStatementRes {
    let excution_res = match statement_type {
        StatementType::StatementInsert { ref row } => exec_insert(&row, cursor),
        StatementType::StatementSelect => exec_select(&mut cursor.table),
    };

    excution_res
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ExecStatementRes {
    ExecSuccess,
    ExecFailure { cause: String },
    ExecExit,
}

fn exec_insert(row: &Row, cursor: &mut Cursor) -> ExecStatementRes {
    let page = Page::new(cursor.table.pager.get_page(cursor.curr_page_num).unwrap());

    if page.cell_count() as usize >= LEAF_NODE_MAX_CELLS {
        return ExecStatementRes::ExecFailure {
            cause: "table full".to_string(),
        };
    }

    match cursor.insert_leaf_page(row.id as usize, row) {
        Ok(v) => v,
        Err(e) => {
            return ExecStatementRes::ExecFailure {
                cause: e.to_string(),
            };
        }
    };

    ExecStatementRes::ExecSuccess
}

fn exec_select(table: &mut Table) -> ExecStatementRes {
    let mut cursor = Cursor::new(table);
    let mut row = Row::new();

    while !cursor.at_table_end {
        match cursor.curr_value() {
            Ok(val) => {
                row.ingest_deserialized(val);
                println!("{}", row);
                cursor.advance();
            }
            Err(e) => {
                return ExecStatementRes::ExecFailure {
                    cause: e.to_string(),
                };
            }
        }
    }

    ExecStatementRes::ExecSuccess
}

pub fn exec_clear() -> ExecStatementRes {
    print!("\x1B[2J\x1B[H");
    io::stdout()
        .flush()
        .expect("failed to flush clear screen ANSI escape codes");

    ExecStatementRes::ExecSuccess
}

pub fn print_constants() -> ExecStatementRes {
    println!("ROW_SIZE: {}\n", ROW_SIZE);
    println!("COMMON_NODE_HEADER_SIZE: {}\n", COMMON_HEADER_SIZE);
    println!("LEAF_NODE_HEADER_SIZE: {}\n", LEAF_NODE_HEADER_SIZE);
    println!("LEAF_NODE_CELL_SIZE: {}\n", LEAF_NODE_CELL_SIZE);
    println!("LEAF_NODE_SPACE_FOR_CELLS: {}\n", LEAF_NODE_SPACE_FOR_CELLS);
    println!("LEAF_NODE_MAX_CELLS: {}\n", LEAF_NODE_MAX_CELLS);

    ExecStatementRes::ExecSuccess
}

pub fn exect_print_btree(cursor: &mut Cursor) -> ExecStatementRes {
    let page_bytes = &mut cursor.table.pager.get_page(0).unwrap();
    let root_page = Page::new(page_bytes);

    println!("Tree:");
    root_page.print_leaf_node();

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
    fn test_meta_command_clear() {
        let result = "_cl".parse::<MetaCmdRes>();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), MetaCmdRes::MetaCmdClear);
    }

    #[test]
    fn test_meta_command_unrecognized() {
        let result = "random".parse::<MetaCmdRes>();
        assert!(result.is_err());
    }
}
