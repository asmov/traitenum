use clap;
use std::path::PathBuf;

use crate::str;

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub module: CommandModules
}

#[derive(clap::Subcommand)]
pub enum CommandModules {
    Workspace(WorkspaceCommandModule),
    Trait(TraitCommandModule)
}

#[derive(clap::Args)]
#[command(about = "Manage traitenum workspaces")]
pub struct WorkspaceCommandModule {
    #[command(subcommand)]
    pub command: WorkspaceCommands
}

#[derive(clap::Args)]
#[command(about = "Manage traitenum traits")]
pub struct TraitCommandModule {
    #[command(subcommand)]
    pub command: TraitCommands 
}

#[derive(clap::Subcommand)]
pub enum WorkspaceCommands {
    #[command(about = "Create a new traitenum workspace containing traits and derive macros")]
    New(WorkspaceCommand),
    #[command(about = "Create new traitenum lib and derive packages in an existing workspace")]
    Init(WorkspaceCommand),
}

#[derive(clap::Subcommand)]
pub enum TraitCommands {
    Add(AddTraitCommand),
}

#[derive(clap::Args)]
pub struct WorkspaceCommand {
    pub workspace_name: String,
     #[arg(long)]
    pub workspace_path: Option<PathBuf>,
     #[arg(long)]
    pub lib_name: Option<String>,
    #[arg(long)]
    pub derive_name: Option<String>,
    #[arg(long, default_value_t = str!("lib"))]
    pub lib_dir: String,
    #[arg(long, default_value_t = str!("derive"))]
    pub derive_dir: String
}

#[derive(clap::Args)]
#[command(about = "Add a new trait and derive macro to an existing traitenum workspace")]
pub struct AddTraitCommand {
    pub trait_name: String,
    #[arg(long)]
    pub lib_path: Option<PathBuf>,
    #[arg(long)]
    pub derive_path: Option<PathBuf>,
}