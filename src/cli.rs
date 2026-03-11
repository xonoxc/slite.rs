use std::io::{self, Write};
use std::process::{self};

use crate::cmd::CLIcommand;
use crate::data::table::Table;
use crate::input_buffer::InputBuffer;
use crate::statements::{MetaCmdRes, exec_statement};

pub fn run() {
    println!("Welcome to slite-rs CLI:");

    let db_file_path = String::from("sample.db");

    let mut main_table = Table::new(&db_file_path);

    let mut command = InputBuffer::new();
    loop {
        next_prompt();
        exec_command(&mut main_table, read_prompt(&mut command).buffer.trim())
    }
}

fn exec_command(table: &mut Table, command: &str) {
    match command.parse::<CLIcommand>() {
        Ok(CLIcommand::Meta(MetaCmdRes::ExitCmd)) => process::exit(0),

        Ok(CLIcommand::Meta(_)) => {
            println!("executing meta command!")
        }

        Ok(CLIcommand::Statement(stmt_type)) => exec_statement(stmt_type, table),

        Err(_) => println!("SINTAX ERROR: Unrecognized command '{}'", command),
    }
}

fn next_prompt() {
    print!("> ");
    io::stdout().flush().expect("Failed to flush prompt");
}

fn read_prompt(input_buffer: &mut InputBuffer) -> &InputBuffer {
    input_buffer.clear();

    let bytes_read = io::stdin()
        .read_line(&mut input_buffer.buffer)
        .expect("failed to read command");

    if bytes_read == 0 {
        process::exit(1);
    }

    input_buffer
}
