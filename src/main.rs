pub mod cli;
pub mod cmd;
pub mod input_buffer;
pub mod row;
pub mod statements;

fn main() {
    cli::run();
}
