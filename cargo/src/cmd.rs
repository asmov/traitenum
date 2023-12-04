use std::{process, path::{PathBuf, Path}};
use anyhow::Context;
use crate::{self as lib, str};

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

pub(crate) fn read_manifest(filepath: &Path) -> anyhow::Result<toml::Value> {
    let contents = std::fs::read_to_string(filepath)?;
    toml::from_str(&contents).map_err(|e| anyhow::format_err!("{}", e.message()))
}

pub(crate) fn find_cargo_workspace_manifest(from_dir: &Path) -> anyhow::Result<(toml::Value, PathBuf)> {
    // if first manifest found is a package, we'll try once more to find a parent workspace
    let mut dir = from_dir;

    while let Ok(manifest_file) = find_cargo_manifest_file(dir) {
        let manifest = read_manifest(&manifest_file)?;
        if manifest.as_table()
                .with_context(|| lib::Errors::InvalidCargoMetadata(str!("workspace"), manifest_file))?
                .contains_key("workspace") {
            return Ok((manifest, from_dir.join("Cargo.toml")))
        }

        dir = match dir.parent() { Some(d) => d, None => break };
    }

    Err(lib::Errors::NoCargoWorkspaceExists(from_dir.into()).into())
}
