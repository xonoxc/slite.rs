use std::io::{self, BufRead, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, mpsc};
use std::{env, thread};

use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook::flag;

use crate::cmd::CLIcommand;
use crate::cursor::Cursor;
use crate::data::table::Table;
use crate::errors::ParseError;
use crate::input_buffer::InputBuffer;
use crate::statements::{
    ExecStatementRes, MetaCmdRes, exec_clear, exec_statement, exect_print_btree, print_constants,
};

pub fn run() {
    println!("Welcome to slite-rs CLI:");

    let shutdown = setup_interrupt_listeners();
    let mut main_table = Table::new(&get_db_file_name().unwrap()).unwrap();
    let mut cursor = Cursor::new(&mut main_table);

    let (sender_end, reciver_end) = mpsc::channel::<InputBuffer>();

    spawn_input_thead(sender_end);
    next_prompt();

    loop {
        if shutdown.load(Ordering::Relaxed) {
            println!("\nshutting down...");
            break;
        }

        match reciver_end.recv_timeout(std::time::Duration::from_millis(100)) {
            Ok(buffer) => {
                match exec_command(buffer.buffer.trim(), &mut cursor) {
                    ExecStatementRes::ExecFailure { cause } => {
                        println!("Error executing command: {}", cause);
                    }
                    ExecStatementRes::ExecExit => break,
                    ExecStatementRes::ExecSuccess => {}
                }

                next_prompt();
            }
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(_) => break,
        };
    }
}

fn exec_command(command: &str, cursor: &mut Cursor) -> ExecStatementRes {
    match command.parse::<CLIcommand>() {
        Ok(CLIcommand::Meta(cmd)) => exect_meta_cmd(cmd, cursor),
        Ok(CLIcommand::Statement(stmt_type)) => exec_statement(stmt_type, cursor),

        Err(_) => ExecStatementRes::ExecFailure {
            cause: "SINTAX ERROR: Unrecognized command".to_string(),
        },
    }
}

fn exect_meta_cmd(meta_cmd: MetaCmdRes, cursor: &mut Cursor) -> ExecStatementRes {
    match meta_cmd {
        MetaCmdRes::ExitCmd => ExecStatementRes::ExecExit,
        MetaCmdRes::MetaCmdClear => exec_clear(),
        MetaCmdRes::MetaCmdConstants => print_constants(),
        MetaCmdRes::MetaCmdBtree => exect_print_btree(cursor),
        _ => ExecStatementRes::ExecFailure {
            cause: "Unrecognized meta cmd".to_string(),
        },
    }
}

fn next_prompt() {
    print!(" > ");
    io::stdout()
        .flush()
        .expect("failed to flush next prompt ANSI escape codes");
}

fn spawn_input_thead(sender_end: Sender<InputBuffer>) {
    thread::spawn(move || {
        let stdin = io::stdin();
        let mut handle = stdin.lock();

        loop {
            let mut command = InputBuffer::new();

            if handle.read_line(&mut command.buffer).is_err() {
                continue;
            }

            if command.buffer.trim().is_empty() {
                continue;
            }

            if sender_end.send(command).is_err() {
                break;
            }
        }
    });
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

    shutdown
}
