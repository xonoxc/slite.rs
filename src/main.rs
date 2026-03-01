use std::{
    io::{self, Write},
    process::{self},
};
pub mod input_buffer;

fn main() {
    println!("Welcome to slite-rs CLI:");

    let mut command = String::new();

    loop {
        next_prompt();
        let command = read_prompt(&mut command);

        if command == ".exit" {
            process::exit(0);
        } else {
            println!("unrecognized command.. {}", command);
        }
    }
}

fn next_prompt() {
    print!("> ");
    io::stdout().flush().expect("Failed to flush prompt");
}

fn read_prompt(input_buffer: &mut String) -> String {
    input_buffer.clear();
    io::stdin()
        .read_line(input_buffer)
        .expect("failed to read command");

    input_buffer.trim().to_string()
}
