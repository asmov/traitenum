use std::process;
use clap::Parser;
use colored::Colorize;
use traitenum_cargo::cli;

fn main() {
    let cli = cli::Cli::parse();
    match traitenum_cargo::run(cli) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("{}{}", "[traitenum] ".red(), e);
            process::exit(1);
        }
    }
}
