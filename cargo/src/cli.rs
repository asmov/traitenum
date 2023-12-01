use clap;
use std::path::PathBuf;


#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands
}

#[derive(clap::Subcommand)]
pub enum Commands {
    New(NewCommand),
    Add(AddCommand)
}

#[derive(clap::Args)]
#[command(about = "Create a new traitenum workspace containing traits and derive macros")]
pub struct NewCommand {
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
pub struct AddCommand {
    pub trait_name: String,
}