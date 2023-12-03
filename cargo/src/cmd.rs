use std::{process, path::{PathBuf, Path}};
use anyhow::Context;
use crate::{self as lib, meta, str};

pub mod workspace;
pub mod enumtrait;

pub use workspace::new::new_workspace;
pub use enumtrait::add::add_trait;

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

fn find_cargo_manifest_file(from_dir: &Path) -> anyhow::Result<PathBuf> {
    let mut current_dir = from_dir.to_owned();

    while current_dir.exists() {
        let cargo_manifest_filepath = current_dir.join("Cargo.toml");
        if cargo_manifest_filepath.exists() {
            return Ok(cargo_manifest_filepath);
        }

        current_dir = current_dir.join("..");
    }

    Err(lib::Errors::NoCargoManifestExists(from_dir.into()).into())
}

pub fn read_manifest(filepath: &Path) -> anyhow::Result<toml::Table> {
    let contents = std::fs::read_to_string(filepath)?;
    toml::from_str(&contents).map_err(|e| anyhow::format_err!("{}", e.message()))
}

fn find_cargo_workspace_manifest(from_dir: &Path) -> anyhow::Result<(toml::Table, PathBuf)> {
    // if first manifest found is a package, we'll try once more to find a parent workspace
    let mut dir = from_dir;

    while let Ok(manifest_file) = find_cargo_manifest_file(dir) {
        let manifest = read_manifest(&manifest_file)?;
        if manifest.contains_key("workspace") {
            if let Some(workspace_table) = manifest["workspace"].as_table() {
                return Ok((workspace_table.to_owned(), from_dir.into()))
            }
        }

        dir = match dir.parent() { Some(d) => d, None => break };
    }

    Err(lib::Errors::NoCargoWorkspaceExists(from_dir.into()).into())
}

fn build_meta(from_dir: &Path) -> anyhow::Result<meta::WorkspaceMeta> {
    let (workspace_manifest, workspace_path) = find_cargo_workspace_manifest(&from_dir)?;

    let mut workspace = meta::build::WorkspaceMeta::new();
    workspace.path(workspace_path);

    let libraries_metadata = workspace_manifest.get("workspace.metadata.traitenum.library")
        .with_context(|| lib::Errors::MissingCargoMetadata(
            str!("workspace.metadata.traitenum.library"), workspace.get_path().to_owned()))?
        .as_array()
        .with_context(|| lib::Errors::InvalidCargoMetadata(
            str!("workspace.metadata.traitenum.library"), workspace.get_path().to_owned()))?;

    let mut libraries: Vec<meta::build::LibraryMeta> = Vec::new();
    let mut i = 0;
    for library_metadata in libraries_metadata {
        let library_metadata = library_metadata.as_table()
            .with_context(|| lib::Errors::InvalidCargoMetadata(
                format!("workspace.metadata.traitenum.library[{}]", i), workspace.get_path().to_owned()))?;
        
        let name = library_metadata.get("name")
            .with_context(|| lib::Errors::MissingCargoMetadata(
                format!("workspace.metadata.traitenum.library[{}].name", i), workspace.get_path().to_owned()))?
            .as_str()
            .with_context(|| lib::Errors::InvalidCargoMetadata(
                format!("workspace.metadata.traitenum.library[{}].name", i), workspace.get_path().to_owned()))?;
    
        let lib_dir = library_metadata.get("lib_dir")
            .with_context(|| lib::Errors::MissingCargoMetadata(
                format!("workspace.metadata.traitenum.library[{}].lib_dir", name), workspace.get_path().to_owned()))?
            .as_str()
            .with_context(|| lib::Errors::InvalidCargoMetadata(
                format!("workspace.metadata.traitenum.library[{}].lib_dir", name), workspace.get_path().to_owned()))?;

        let derive_dir = library_metadata.get("derive_dir")
            .with_context(|| lib::Errors::MissingCargoMetadata(
                format!("workspace.metadata.traitenum.library[{}].derive_dir", name), workspace.get_path().to_owned()))?
            .as_str()
            .with_context(|| lib::Errors::InvalidCargoMetadata(
                format!("workspace.metadata.traitenum.library[{}].derive_dir", name), workspace.get_path().to_owned()))?;

        let mut library = meta::build::LibraryMeta::new();
        library.name(name.to_owned());
        library.lib_dir(lib_dir.to_owned());
        library.derive_dir(derive_dir.to_owned());
        libraries.push(library);
        i += 1;
    }

    for library in &mut libraries {
        let lib_path = workspace.get_lib_path(&library);
        let derive_path = workspace.get_derive_path(&library);

        let manifest = read_manifest(&lib_path.join("Cargo.toml"))?;
        let lib_name = manifest.get("package.name")
            .with_context(|| lib::Errors::MissingCargoMetadata(str!("package.name"), lib_path.to_owned()))?
            .as_str()
            .with_context(|| lib::Errors::InvalidCargoMetadata(str!("package.name"), lib_path.to_owned()))?
            .to_owned();

        let manifest = read_manifest(&derive_path.join("Cargo.toml"))?;
        let derive_name = manifest.get("package.name")
            .with_context(|| lib::Errors::MissingCargoMetadata(str!("package.name"), lib_path.to_owned()))?
            .as_str()
            .with_context(|| lib::Errors::InvalidCargoMetadata(str!("package.name"), lib_path.to_owned()))?
            .to_owned();

        library.lib_name(lib_name);
        library.derive_name(derive_name);
    }

    workspace.libraries(libraries);
    Ok(workspace.build())
}

