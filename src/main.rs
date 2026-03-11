pub mod cli;
pub mod cmd;
pub mod data;
pub mod errors;
pub mod input_buffer;
pub mod pager;
pub mod statements;

fn main() {
    cli::run();
}
