use std::path::PathBuf;
use anyhow;
use colored::Colorize;
use thiserror;

pub mod meta;
pub mod cli;
pub mod cmd;

#[macro_export]
macro_rules! str { ($s:literal) => { String::from($s) }; }

pub fn log(quiet: bool, msg: &str) {
    if !quiet {
        println!("{}{}", "[traitenum] ".cyan(), msg);
    }
}

pub fn log_warn(msg: &str) {
    eprintln!("{}{}", "[traitenum] ".yellow(), msg);
}

pub fn log_success(quiet: bool, msg: &str) {
    if !quiet {
        println!("{}{}", "[traitenum] ".green(), msg);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Errors {
    #[error("Unable to parse source-code. {0}: {1}")]
    SourceParsing(String, PathBuf),
    #[error("Invalid argument for `{0}` ({1}): {2}")]
    InvalidArgument(String, String, String),
    #[error("Trait already exists in library `{1}`: {0}")]
    DuplicateTrait(String, String),
    #[error("Trait does not exist in library `{1}`: {0}")]
    UnknownTrait(String, String),
    #[error("Misconfigured cargo metadata: {0}")]
    MisconfiguredCargoMetadata(String),
    #[error("Missing --library-name argument (Multiple libraries exist)")]
    AmbiguousLibrary,
    #[error("Library not found: {0}")]
    LibraryNotFound(String),
    #[error("A cargo manifest already exists for path (Try `init` to add workspace members): {0}")]
    CargoManifestExists(PathBuf),
    #[error("A cargo manifest cannot be found for path: {0}")]
    NoCargoManifestExists(PathBuf),
    #[error("Invalid metadata for `{0}` in cargo manifest dir: {1}")]
    InvalidCargoMetadata(String, PathBuf),
    #[error("Unable to parse cargo manifest: {0}")]
    InvalidCargoManifest(PathBuf),
    #[error("Unable to parse cargo manifest for key `{0}`: {1}")]
    InvalidCargoManifestKey(String, PathBuf),
    #[error("Missing metadata for `{0}` in cargo manifest dir: {1}")]
    MissingCargoMetadata(String, PathBuf),
    #[error("A cargo workspace cannot be found for path: {0}")]
    NoCargoWorkspaceExists(PathBuf),
    #[error("The cargo manifest is not a workspace: {0}")]
    CargoManifestNotWorkspace(PathBuf),
    #[error("Unable to run command: cargo")]
    CargoRunError(),
    #[error("Unable to run command: rustfmt")]
    RustfmtRunError(),
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
            cli::WorkspaceCommands::New(args) => cmd::new_workspace(args, cli.quiet),
            cli::WorkspaceCommands::Init(args) => cmd::init_workspace(args, cli.quiet),
        },
        cli::CommandModules::Trait(module) => match module.command {
            cli::TraitCommands::Add(args) => cmd::add_trait(args, cli.quiet, true),
            cli::TraitCommands::Remove(args) => cmd::remove_trait(args, cli.quiet),
        }
    }
}
