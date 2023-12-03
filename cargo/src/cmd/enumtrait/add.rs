use std::{env, path::{PathBuf, Path}};
use anyhow::Context;
use crate::{self as lib, cmd::{self, find_cargo_manifest_file}, cli, meta, str};

pub fn add_trait(args: cli::AddTraitCommand) -> anyhow::Result<()> {
    let dir = if let Some(workspace_path) = args.workspace_path {
        workspace_path
    } else {
        env::current_dir()?
    };

    let workspace = meta::build(&dir)?;
    
    Ok(())
}

