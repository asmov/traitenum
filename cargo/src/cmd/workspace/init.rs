use std::{env, path::{PathBuf, Path}};
use crate::{self as lib, cli, cmd, meta};

pub fn init_workspace(mut args: cli::InitWorkspaceCommand) -> anyhow::Result<()> {
    // clarify to the user that library.lib_name and library_name are the same
    // todo: remove lib_name from the common
    if args.library.lib_name.is_some() && args.library_name != args.library.lib_name.unwrap() {
        lib::log_warn("Using preferred `<LIBRARY_NAME>` argument instead of `--lib-name`")
    } else {
        args.library.lib_name = Some(args.library_name.clone());
    }

    if let Some(ref workspace_path) = args.library.workspace_path {
        if workspace_path.is_relative() {
            args.library.workspace_path = Some(PathBuf::from(env::current_dir().unwrap())
                .join(workspace_path));
        }
    } else {
        args.library.workspace_path = Some(PathBuf::from(env::current_dir().unwrap())
            .join(&args.library_name));
    }

    if args.library.derive_name.is_none() {
        args.library.derive_name = Some(format!("{}-{}", args.library_name, "derive"));
    }

    // Throw an error if `new` should be used instead of `init`.
    let workspace_path = args.library.workspace_path.as_ref().unwrap();
    let workspace_manifest_filepath = cmd::find_cargo_manifest_file(&workspace_path)?;
    let workspace_manifest = cmd::read_workspace_manifest(&workspace_manifest_filepath)?;

    /*lib::log("Creating lib package ...");
    make_lib(&args)?;
    lib::log("Creating derive package ...");
    make_derive(&args)?;
    lib::log("Updating workspace ...");
    update_workspace(&args)?;
    lib::log("Configuring lib package ...");
    config_lib(&args)?;
    lib::log("Configuring derive package ...");
    config_derive(&args)?;
    lib::log("Building workspace ...");
    build_workspace(&args)?;
    lib::log("Testing workspace ...");
    test_workspace(&args)?;
    lib::log_success("Your traitenum workspace is ready.");
    */

    Ok(())

}