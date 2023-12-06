use std::{fs, process, env, path::Path};
use anyhow::Context;
use syn;
use quote::{self, ToTokens};
use convert_case::{self as case, Casing};
use crate::{self as lib, cli, meta::{self, LibraryMeta}, str, cmd};


pub fn remove_trait(args: cli::RemoveTraitCommand) -> anyhow::Result<()> {
    let dir = if let Some(ref workspace_path) = args.module.workspace_path {
        workspace_path.to_owned()
    } else {
        env::current_dir()?
    };

    let workspace = meta::build(&dir)?;

    // find the library
    let library = if workspace.libraries().len() == 1 {
        workspace.libraries().first().unwrap()
    } else if workspace.libraries().len() > 1 {
        let library_name = match &args.module.library_name {
            Some(name) => name,
            None => anyhow::bail!(lib::Errors::AmbiguousLibrary)
        };

        workspace.libraries().iter().find(|lib| lib.name() == library_name)
            .context(lib::Errors::LibraryNotFound(library_name.to_owned()))?
    } else {
        anyhow::bail!(lib::Errors::MisconfiguredCargoMetadata(str!("No traitenum libraries found")))
    };

    if library.traits().iter().find(|t| t.crate_path() == args.module.trait_crate_path).is_none() {
        anyhow::bail!(lib::Errors::UnknownTrait(args.module.trait_crate_path, library.name().to_owned()))
    }

    /*lib::log("Removing trait from lib package ...");
    rm_lib_trait();
    lib::log("Removing macro from derive package ...");
    rm_derive_macro();
    lib::log("Removing integration test from derive package ...");
    rm_derive_integration_test();
    lib::log("Testing workspace ...");
    test_workspace();
    lib::log_success("The enumtrait has been removed.");
    */

    Ok(())
}