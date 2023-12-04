use std::{fs, process, env};
use anyhow::Context;
use syn;
use quote::{self, ToTokens};
use crate::{self as lib, cli, meta::{self, LibraryMeta}, str};

pub fn add_trait(args: cli::AddTraitCommand) -> anyhow::Result<()> {
    let dir = if let Some(ref workspace_path) = args.workspace_path {
        workspace_path.to_owned()
    } else {
        env::current_dir()?
    };

    let workspace = meta::build(&dir)?;

    // find the library
    let library = if workspace.libraries().len() == 1 {
        workspace.libraries().first().unwrap()
    } else if workspace.libraries().len() > 1 {
        let library_name = match &args.library_name {
            Some(l) => l,
            None => anyhow::bail!(lib::Errors::AmbiguousLibrary)
        };
        workspace.libraries().iter().find(|l| l.name() == library_name)
            .context(lib::Errors::LibraryNotFound(library_name.to_owned()))?
    } else {
        anyhow::bail!(lib::Errors::MisconfiguredCargoMetadata(str!("No traitenum libraries found")))
    };

    if library.traits().iter().find(|t| t.crate_path() == args.trait_crate_path).is_some() {
        anyhow::bail!(lib::Errors::DuplicateTrait(args.trait_crate_path, library.name().to_owned()))
    }

    lib::log("Adding lib trait ...");
    add_lib_trait(&args, &workspace, library)?;
    //lib::log("Adding derive macro ...");
    //add_derive_macro(&args, &workspace, library)?;
    //lib::log("Updating cargo manifest ...");
    //update_cargo_manifest(&args, &workspace, library)?;
    //lib::log_success("Your enumtrait is ready.");
   
    Ok(())
}

fn add_lib_trait(
    args: &cli::AddTraitCommand,
    workspace: &meta::WorkspaceMeta,
    library: &LibraryMeta
) -> anyhow::Result<()> {
    let lib_src_path = workspace.lib_path(library).join("src").join("lib.rs");
    let lib_src = std::fs::read_to_string(&lib_src_path).unwrap();
    let trait_crate_path: syn::Path = syn::parse_str(&args.trait_crate_path).unwrap();
    let trait_item = trait_item(trait_crate_path);

    let mut lib_src_file = syn::parse_file(&lib_src).unwrap();
    lib_src_file.items.push(trait_item);
    fs::write(&lib_src_path, lib_src_file.to_token_stream().to_string())?;

    process::Command::new("rustfmt")
        .arg(lib_src_path.to_str().unwrap())
        .output()
        .expect("Unable to run: rustfmt");
 
    Ok(())
}

fn add_derive_macro(
    args: &cli::AddTraitCommand,
    workspace: &meta::WorkspaceMeta,
    library: &LibraryMeta
) -> anyhow::Result<()> {
    todo!()
}

fn update_cargo_manifest(
    args: &cli::AddTraitCommand,
    workspace: &meta::WorkspaceMeta,
    library: &LibraryMeta
) -> anyhow::Result<()> {
    todo!()
}

fn trait_item(trait_crate_path: syn::Path) -> syn::Item {
    let trait_ident = &trait_crate_path.segments.last().unwrap().ident;

    let item = quote::quote!{
        #[enumtrait(#trait_crate_path)]
        pub trait #trait_ident {
            #[enumtrait::Str(preset(Variant))]
            fn name(&self) -> &'static str;

            #[enumtrait::Num(preset(Ordinal))]
            fn ordinal(&self) -> usize;
        }
    };

    syn::parse2(item).unwrap()
}

