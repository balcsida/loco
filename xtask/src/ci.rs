use crate::errors::Result;
use crate::utils;
use duct::cmd;
use std::path::Path;
use std::path::PathBuf;
use std::process::Output;

const FMT_TEST: [&str; 3] = ["test", "--all-features", "--all"];
const FMT_ARGS: [&str; 4] = ["fmt", "--all", "--", "--check"];
const FMT_CLIPPY: [&str; 8] = [
    "clippy",
    "--",
    "-W",
    "clippy::pedantic",
    "-W",
    "rust-2021-compatibility",
    "-W",
    "rust-2018-idioms",
];

#[derive(Default, Debug)]
pub struct RunResults {
    pub path: PathBuf,
    pub fmt: bool,
    pub clippy: bool,
    pub test: bool,
}

impl RunResults {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.fmt && self.clippy && self.test
    }
}

/// Run CI on all Loco resources (lib, cli, starters, examples, etc.).
///
/// # Errors
/// when could not run ci on the given resource
///
pub fn all_resources(base_dir: &Path) -> Result<Vec<RunResults>> {
    let mut result = vec![];
    result.push(run(base_dir).expect("loco lib mast be tested"));
    result.extend(inner_folders(&base_dir.join(utils::FOLDER_EXAMPLES))?);
    result.extend(inner_folders(&base_dir.join(utils::FOLDER_STARTERS))?);
    result.extend(inner_folders(&base_dir.join(utils::FOLDER_LOCO_CLI))?);

    Ok(result)
}

/// Run CI on inner folders.
///
/// For example, run CI on all examples/starters folders dynamically by selecting the first root folder and running CI one level down.
///
/// # Errors
/// when could not get cargo folders
pub fn inner_folders(root_folder: &Path) -> Result<Vec<RunResults>> {
    let cargo_projects = utils::get_cargo_folders(root_folder)?;
    let mut results = vec![];

    for project in cargo_projects {
        if let Some(res) = run(&project) {
            results.push(res);
        }
    }
    Ok(results)
}

/// Run the entire CI flow on the given folder path.
///
/// Returns `None` if it is not a Rust folder.
#[must_use]
pub fn run(dir: &Path) -> Option<RunResults> {
    if dir.join("Cargo.toml").exists() {
        Some(RunResults {
            path: dir.to_path_buf(),
            fmt: cargo_fmt(dir).is_ok(),
            clippy: cargo_clippy(dir).is_ok(),
            test: cargo_test(dir).is_ok(),
        })
    } else {
        None
    }
}

/// Run cargo test on the given directory.
fn cargo_test(dir: &Path) -> Result<Output> {
    println!(
        "Running `cargo {}` in folder {}",
        FMT_TEST.join(" "),
        dir.display()
    );
    Ok(cmd("cargo", FMT_TEST.as_slice()).dir(dir).run()?)
}

/// Run cargo fmt on the given directory.
fn cargo_fmt(dir: &Path) -> Result<Output> {
    println!(
        "Running `cargo {}` in folder {}",
        FMT_ARGS.join(" "),
        dir.display()
    );
    Ok(cmd("cargo", FMT_ARGS.as_slice()).dir(dir).run()?)
}

/// Run cargo clippy on the given directory.
fn cargo_clippy(dir: &Path) -> Result<Output> {
    println!(
        "Running `cargo {}` in folder {}",
        FMT_CLIPPY.join(" "),
        dir.display()
    );
    Ok(cmd("cargo", FMT_CLIPPY.as_slice()).dir(dir).run()?)
}
