use anyhow;
use colored::Colorize;
use thiserror;

macro_rules! str { ($s:literal) => { String::from($s) }; }

pub fn log(msg: &str) {
    println!("{}{}", "[traitenum] ".cyan(), msg);
}

pub fn log_success(msg: &str) {
    println!("{}{}", "[traitenum] ".green(), msg);
}

pub fn snake_name(name: &str) -> String {
    name.replace("-", "_")
}

#[derive(Debug, thiserror::Error)]
pub enum Errors {
    #[error("Unable to run command: cargo")]
    CargoRunError(),
    #[error("Command `cargo new` failed: {0}")]
    CargoNewError(String),
    #[error("Command `cargo add` failed for `{0}`: {1}")]
    CargoAddError(String, String),
    #[error("Command `cargo {0}` failed")]
    CargoError(String),
}

pub mod cli;

pub mod cmd {
    use std::process;

    fn quote_error(errmsg: String) -> String {
        let errmsg = errmsg.replace("error: ", "");
        if let Some(offset) = errmsg.find("\n") {
            errmsg[0 .. offset].to_owned()
        } else {
            errmsg
        }
    }

    fn quote_error_output(output: process::Output) -> String {
        quote_error(String::from_utf8(output.stderr).unwrap())
    }

    pub mod new;
    pub use new::new_workspace;
}

pub fn run(cli: cli::Cli) -> anyhow::Result<()> {
    match cli.command {
        cli::Commands::New(args) => cmd::new_workspace(args)
    }
}

