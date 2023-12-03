use std::path::PathBuf;
use anyhow;
use colored::Colorize;
use thiserror;

pub mod meta;
pub mod cli;
pub mod cmd;

#[macro_export]
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
    #[error("Invalid argument for `{0}` ({1}): {2}")]
    InvalidArgument(String, String, String),
    #[error("A cargo manifest already exists for path (Try `init` to add workspace members): {0}")]
    CargoManifestExists(PathBuf),
    #[error("A cargo manifest cannot be found for path: {0}")]
    NoCargoManifestExists(PathBuf),
    #[error("Invalid metadata for `{0}` in cargo manifest: {0}")]
    InvalidCargoMetadata(String, PathBuf),
    #[error("Missing metadata for `{0}` in cargo manifest: {0}")]
    MissingCargoMetadata(String, PathBuf),
    #[error("A cargo workspace cannot be found for path: {0}")]
    NoCargoWorkspaceExists(PathBuf),
     #[error("Unable to run command: cargo")]
    CargoRunError(),
    #[error("Command `cargo new` failed: {0}")]
    CargoNewError(String),
    #[error("Command `cargo add` failed for `{0}`: {1}")]
    CargoAddError(String, String),
    #[error("Command `cargo {0}` failed")]
    CargoError(String),
}

pub fn run(cli: cli::Cli) -> anyhow::Result<()> {
    match cli.module {
        cli::CommandModules::Workspace(module) => match module.command {
            cli::WorkspaceCommands::New(args) => cmd::new_workspace(args),
            cli::WorkspaceCommands::Init(args) => todo!(),
            
        },
        cli::CommandModules::Trait(module) => match module.command {
            cli::TraitCommands::Add(args) => cmd::add_trait(args),
        }
    }
}
