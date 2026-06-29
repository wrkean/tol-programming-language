#![allow(unused)]

use clap::Parser;
use std::path::PathBuf;

mod lexer;
mod prelude;
mod token;

fn main() {
    let args = Args::parse();
}

#[derive(Parser)]
struct Args {
    /// Path to the input file
    #[arg(help("Path na nagtuturo sa input file"))]
    input: PathBuf,
}
