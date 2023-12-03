use std::{path, env, path::{PathBuf}};
use crate::{self as lib, cmd, cli};

pub fn add_trait(mut args: cli::AddTraitCommand) -> anyhow::Result<()> {
    // if either path is missing, we have to determine it using generated manifest metadata
    if args.lib_path.is_none() || args.derive_path.is_none() {
        if let Some(lib_path) = args.lib_path {

        } else if let Some(derive_path) = args.derive_path {

        }

        /*let cargo_manifest_filepath = match cmd::find_cargo_manifest(&PathBuf::from(env::current_dir()?)) {
            Some(filepath) => filepath,
            None => anyhow::bail!(lib::Errors::NoCargoManifestExists(()))
        }*/
        
    } 

    Ok(())
}

