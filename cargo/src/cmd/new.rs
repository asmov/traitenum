use std::{env, fs, process, path::{PathBuf, Path}};
use anyhow::{self, Context};
use crate::{cli, cmd, Errors, log, log_success, snake_name};

pub fn new_workspace(mut args: cli::NewCommand) -> anyhow::Result<()> {
    if let Some(ref workspace_path) = args.workspace_path {
        if workspace_path.is_relative() {
            args.workspace_path = Some(PathBuf::from(env::current_dir().unwrap())
                .join(workspace_path));
        }
    } else {
        args.workspace_path = Some(PathBuf::from(env::current_dir().unwrap())
            .join(&args.workspace_name));
    }

    if args.lib_name.is_none() {
        args.lib_name = Some(args.workspace_name.clone());
    }

    if args.derive_name.is_none() {
        args.derive_name = Some(format!("{}-{}", args.workspace_name, "derive"));
    }

    log("Creating workspace ...");
    make_workspace(&args)?;
    log("Creating lib package ...");
    make_lib(&args)?;
    log("Creating derive package ...");
    make_derive(&args)?;
    log("Configuring lib package ...");
    config_lib(&args)?;
    log("Configuring derive package ...");
    config_derive(&args)?;
    log("Building workspace ...");
    build_workspace(&args)?;
    log("Testing workspace ...");
    test_workspace(&args)?;

    log_success("Your traitenum workspace is ready.");
    Ok(())
}

const WORKSPACE_MANIFEST_TEMPLATE: &'static str =
r#"[workspace]
resolver = "2"
members = [ "%{LIB_DIR}%", "%{DERIVE_DIR}%" ]
"#;

fn make_workspace(args: &cli::NewCommand) -> anyhow::Result<()> {
    let workspace_path = args.workspace_path.as_ref().unwrap();

    let cmdout = cargo_new(workspace_path, None)?;
    if !cmdout.status.success() {
        anyhow::bail!(Errors::CargoNewError(cmd::quote_error_output(cmdout)))
    }

    fs::remove_dir_all(workspace_path.join("src"))?;

    let workspace_manifest = WORKSPACE_MANIFEST_TEMPLATE
        .replace("%{LIB_DIR}%", &args.lib_dir)
        .replace("%{DERIVE_DIR}%", &args.derive_dir);

    fs::write(workspace_path.join("Cargo.toml"), workspace_manifest)?;

    Ok(())
}

const LIB_MANIFEST_TEMPLATE: &'static str =
r#"[package]
name = "%{LIB_NAME}%"
version = "0.1.0"
edition = "2021"
"#;

const LIB_SRC_TEMPLATE: &'static str =
r#"use traitenum::enumtrait;

#[enumtrait(crate::MyTrait)]
pub trait MyTrait {
    #[enumtrait::Str()]
    fn nickname(&self) -> &'static str;
    #[enumtrait::Num(preset(Ordinal))]
    fn ordinal(&self) -> usize;
}
"#;

fn make_lib(args: &cli::NewCommand) -> anyhow::Result<()> {
    let lib_path = args.workspace_path.as_ref().unwrap().join(&args.lib_dir);
    let lib_name = args.lib_name.as_ref().unwrap();

    let cmdout = cargo_new(&lib_path, Some(lib_name))?;
    if !cmdout.status.success() {
        anyhow::bail!(Errors::CargoNewError(cmd::quote_error_output(cmdout)))
    }

    let lib_manifest = LIB_MANIFEST_TEMPLATE
        .replace("%{LIB_NAME}%", lib_name);

    fs::write(lib_path.join("Cargo.toml"), lib_manifest)?;
    fs::write(lib_path.join("src").join("lib.rs"), LIB_SRC_TEMPLATE)?;

    Ok(())
}

const DERIVE_MANIFEST_TEMPLATE: &'static str =
r#"[package]
name = "%{DERIVE_NAME}%"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true
"#;

const DERIVE_SRC_TEMPLATE: &'static str =
r#"traitenum_lib::gen_require!(%{LIB_CRATE_NAME}%, %{DERIVE_CRATE_NAME}%);

traitenum_lib::gen_derive_macro!(MyTraitEnum, derive_traitenum_mytrait, traitlib::TRAITENUM_MODEL_BYTES_MYTRAIT);
"#;


const DERIVE_SAMPLE_TEST_TEMPLATE: &'static str =
r#"
#[cfg(test)]
mod tests {
    use %{LIB_CRATE_NAME}%::MyTrait;

    #[test]
    fn mytrait() {
        #[derive(%{DERIVE_CRATE_NAME}%::MyTraitEnum)]
        enum MyEnum {
            #[traitenum(nickname("a"))]
            Alpha,
            #[traitenum(nickname("b"))]
            Bravo,
            #[traitenum(nickname("c"))]
            Charlie
        }

        assert_eq!("a", MyEnum::Alpha.nickname());
        assert_eq!("b", MyEnum::Bravo.nickname());
        assert_eq!("c", MyEnum::Charlie.nickname());

        assert_eq!(0, MyEnum::Alpha.ordinal());
        assert_eq!(1, MyEnum::Bravo.ordinal());
        assert_eq!(2, MyEnum::Charlie.ordinal());
    }
}
"#;

fn make_derive(args: &cli::NewCommand) -> anyhow::Result<()> {
    let derive_path = args.workspace_path.as_ref().unwrap().join(&args.derive_dir);
    let derive_name = args.derive_name.as_ref().unwrap();
    let lib_name = args.lib_name.as_ref().unwrap();

    let cmdout = cargo_new(&derive_path, Some(derive_name))?;
    if !cmdout.status.success() {
        anyhow::bail!(Errors::CargoNewError(cmd::quote_error_output(cmdout)))
    }

    let derive_manifest = DERIVE_MANIFEST_TEMPLATE
        .replace("%{DERIVE_NAME}%", derive_name);

    fs::write(derive_path.join("Cargo.toml"), derive_manifest)?;

    let derive_src = DERIVE_SRC_TEMPLATE
        .replace("%{LIB_CRATE_NAME}%", &snake_name(lib_name))
        .replace("%{DERIVE_CRATE_NAME}%", &snake_name(derive_name));

    fs::write(derive_path.join("src").join("lib.rs"), derive_src)?;

    let derive_sample_test = DERIVE_SAMPLE_TEST_TEMPLATE
        .replace("%{LIB_CRATE_NAME}%", &snake_name(lib_name))
        .replace("%{DERIVE_CRATE_NAME}%", &snake_name(derive_name));

    fs::create_dir_all(derive_path.join("tests"))?;
    fs::write(derive_path.join("tests").join("mytrait.rs"), derive_sample_test)?;

    Ok(())
}

fn config_lib(args: &cli::NewCommand) -> anyhow::Result<()> {
    let lib_path = args.workspace_path.as_ref().unwrap().join(&args.lib_dir);

    //todo
    let traitenum_crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("macro");

    cargo_add(&lib_path, None, Some(&traitenum_crate_path))?;

    Ok(())
}

fn config_derive(args: &cli::NewCommand) -> anyhow::Result<()> {
    let derive_path = args.workspace_path.as_ref().unwrap().join(&args.derive_dir);
    let lib_name = args.lib_name.as_ref().unwrap();

    //todo
    let traitenum_lib_crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("lib");

    cargo_add(&derive_path, Some("proc-macro2"), None)?;
    cargo_add(&derive_path, None, Some(&traitenum_lib_crate_path))?;
    cargo_add(&derive_path, Some(lib_name), None)?;

    Ok(())
}

fn build_workspace(args: &cli::NewCommand) -> anyhow::Result<()> {
    let workspace_path = args.workspace_path.as_ref().unwrap();

    env::set_current_dir(workspace_path)?;
    let output = process::Command::new("cargo")
        .arg("build")
        .output()
        .context(Errors::CargoRunError())?;

    if !output.status.success() {
        anyhow::bail!(Errors::CargoError(str!("build")))
    }

    Ok(())
}

fn test_workspace(args: &cli::NewCommand) -> anyhow::Result<()> {
    let workspace_path = args.workspace_path.as_ref().unwrap();

    env::set_current_dir(workspace_path)?;
    let output = process::Command::new("cargo")
        .arg("test")
        .output()
        .context(Errors::CargoRunError())?;

    if !output.status.success() {
        anyhow::bail!(Errors::CargoError(str!("test")))
    }

    Ok(())
}

fn cargo_new(path: &Path, name: Option<&str>) -> anyhow::Result<process::Output> {
    let mut cmd = process::Command::new("cargo");
    cmd.args([ "-q", "new", "--lib" ]);

    if let Some(name) = name {
        cmd.args([ "--name", &name ]);
    }
    
    let output = cmd
        .arg(path.to_str().unwrap())
        .output()
        .context(Errors::CargoRunError())?;

    if !output.status.success() {
        anyhow::bail!(Errors::CargoNewError(cmd::quote_error_output(output)))
    }

    Ok(output)
}


fn cargo_add(manifest_dir: &PathBuf, name: Option<&str>, path: Option<&Path>) -> anyhow::Result<process::Output> {
    let mut cmd = process::Command::new("cargo");
    cmd.args([
        "-q",
        "add",
        "--manifest-path",
        manifest_dir.join("Cargo.toml").to_str().unwrap() ]);

    let target;
    if let Some(name) = name {
        target = name;
        cmd.arg(&name);
    } else if let Some(path) = path {
        target = path.to_str().unwrap();
        cmd.args([ "--path", &target ]);
    } else {
        unreachable!("Neither name nor path was passed as a parameter");
    }
    
    let output = cmd
        .output()
        .context(Errors::CargoRunError())?;

    if !output.status.success() {
        anyhow::bail!(Errors::CargoAddError(target.to_string(), cmd::quote_error_output(output)))
    }

    Ok(output)
}

