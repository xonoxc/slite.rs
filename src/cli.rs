use std::io::{self, Write};
use std::process::{self};

use crate::cmd::CLIcommand;
use crate::input_buffer::InputBuffer;
use crate::statements::{MetaCmdRes, exec_statement};

pub fn run() {
    println!("Welcome to slite-rs CLI:");

    let mut command = InputBuffer::new();
    loop {
        next_prompt();
        exec_command(read_prompt(&mut command).buffer.trim())
    }
}

fn exec_command(command: &str) {
    match command.parse::<CLIcommand>() {
        /*
         *  if exit meta command
         * **/
        Ok(CLIcommand::Meta(MetaCmdRes::ExitCmd)) => process::exit(0),

        /*
         *  if any other meta command
         * **/
        Ok(CLIcommand::Meta(_)) => {
            println!("executing meta command!")
        }

        /*
         * if a statement
         * **/
        Ok(CLIcommand::Statement(stmt_type)) => exec_statement(stmt_type),

        /*
         * invalid command
         * **/
        Err(_) => println!("Invalid command"),
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
