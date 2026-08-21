mod cli;
mod falz;
mod jit;
mod mapper;
mod parser;
mod target;

use cli::Cli;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    Cli::run(args);
}

