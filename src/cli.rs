use std::env;
use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook::flag;

use crate::cmd::CLIcommand;
use crate::data::table::Table;
use crate::errors::ParseError;
use crate::input_buffer::InputBuffer;
use crate::statements::{ExecStatementRes, MetaCmdRes, exec_clear, exec_statement};

pub fn run() {
    println!("Welcome to slite-rs CLI:");

    let shutdown = setup_interrupt_listeners();

    let mut main_table = Table::new(&get_db_file_name().unwrap());

    let mut command = InputBuffer::new();

    loop {
        if shutdown.load(Ordering::Relaxed) {
            println!("\nshutting down...");
            break;
        }
        /*
         * next prompt >
         * */
        next_prompt();

        /*
         * read prompt
         * **/
        let read_prompt = read_prompt(&mut command);

        let input_buffer = match read_prompt {
            Some(v) => v,
            None => continue,
        };

        match exec_command(&mut main_table, input_buffer.buffer.trim()) {
            ExecStatementRes::ExecFailure { cause } => {
                println!("Error executing command: {}", cause);
            }
            ExecStatementRes::ExecExit => break,
            ExecStatementRes::ExecSuccess => {}
        }
    }
}

fn exec_command(table: &mut Table, command: &str) -> ExecStatementRes {
    match command.parse::<CLIcommand>() {
        Ok(CLIcommand::Meta(cmd)) => exect_meta_cmd(cmd),

        Ok(CLIcommand::Statement(stmt_type)) => exec_statement(stmt_type, table),

        Err(_) => ExecStatementRes::ExecFailure {
            cause: "SINTAX ERROR: Unrecognized command".to_string(),
        },
    }
}

fn exect_meta_cmd(meta_cmd: MetaCmdRes) -> ExecStatementRes {
    match meta_cmd {
        MetaCmdRes::ExitCmd => ExecStatementRes::ExecExit,
        MetaCmdRes::MetaCmdClear => exec_clear(),
        _ => ExecStatementRes::ExecFailure {
            cause: "Unrecognized meta cmd".to_string(),
        },
    }
}

fn next_prompt() {
    print!("> ");
    io::stdout()
        .flush()
        .expect("failed to flush next prompt ANSI escape codes");
}

fn read_prompt(input_buffer: &mut InputBuffer) -> Option<&InputBuffer> {
    input_buffer.clear();

    let bytes_read = match io::stdin().read_line(&mut input_buffer.buffer) {
        Ok(0) => return None,
        Ok(bytes) => bytes,
        Err(e) => {
            if e.kind() == io::ErrorKind::Interrupted {
                return None;
            }
            return None;
        }
    };

    if bytes_read == 0 {
        return None;
    }

    Some(input_buffer)
}

fn get_db_file_name() -> Result<String, ParseError> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err(ParseError::ArgsPassError {
            arg: "db filename not provided",
        });
    }

    let db_file_path = String::from(&args[1]);

    Ok(db_file_path)
}

/*
*
* setup an infinite SIGINT and SIGTERM listener
* on the other thead using a shared pointer where shudown clone(another refrenece to the same
* atomic value) get's updated on the other thread and you get the bool on the main one.
* **/
fn setup_interrupt_listeners() -> Arc<AtomicBool> {
    let shutdown = Arc::new(AtomicBool::new(false));

    flag::register(SIGINT, shutdown.clone()).unwrap();
    flag::register(SIGTERM, shutdown.clone()).unwrap();

    flag::register_conditional_shutdown(SIGINT, 1, shutdown.clone()).unwrap();

    shutdown
}
