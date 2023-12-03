use std::{process, path::PathBuf};

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

fn find_cargo_manifest(from_dir: &PathBuf) -> Option<PathBuf> {
    let mut current_dir = from_dir.to_owned();

    while current_dir.exists() {
        let cargo_manifest_filepath = current_dir.join("Cargo.toml");
        if cargo_manifest_filepath.exists() {
            return Some(cargo_manifest_filepath);
        }

        current_dir = current_dir.join("..");
    }

    None
}

