use crate::{
    ci,
    errors::{Error, Result},
    out, utils,
};
use cargo_metadata::semver::Version;
use lazy_static::lazy_static;
use regex::Regex;
use std::io::{Read, Write};
use std::path::Path;
use std::{fs, path::PathBuf};

lazy_static! {
    /// Regular expression for replacing the version in the root package's Cargo.toml file.
    static ref REPLACE_LOCO_LIB_VERSION_: Regex = Regex::new(
        r#"(?P<name>name\s*=\s*".+\s+version\s*=\s*")(?P<version>[0-9]+\.[0-9]+\.[0-9]+)"#
    )
    .unwrap();

    /// Regular expression for updating the version in loco-rs package dependencies in Cargo.toml files.
    static ref REPLACE_LOCO_PACKAGE_VERSION: Regex =
        Regex::new(r#"loco-rs = \{ (version|path) = "[^"]+""#).unwrap();

}

/// Bump all necessary loco resources with the given version.
///
/// # Errors
/// Returns an error when it could not update one of the resources.
pub fn bump(base_dir: &Path, new_version: &Version) -> Result<()> {
    root_package(base_dir, new_version)?;
    println!("Bump Loco lib updated successfully");
    update_starters(
        base_dir,
        "loco-rs = { path = \"../../\"",
        Some("loco-rs = { path = \"../../../\""),
    )?;

    println!("Testing starters CI");
    let starter_projects: Vec<ci::RunResults> =
        ci::inner_folders(&base_dir.join(utils::FOLDER_STARTERS))?;

    println!("Starters CI results:");
    println!("{}", out::ci_results(&starter_projects));
    for starter in &starter_projects {
        if !starter.is_valid() {
            return Err(Error::Message(format!(
                "starter {} ins not passing the CI",
                starter.path.display()
            )));
        }
    }

    update_starters(
        base_dir,
        &format!("loco-rs = {{ version = \"{new_version}\""),
        None,
    )?;
    println!("Bump loco starters finished successfully");

    Ok(())
}

/// Bump the version of the loco library in the root package's Cargo.toml file.
///
/// # Errors
/// Returns an error when it could not parse the loco Cargo.toml file or has an error updating the file.
pub fn root_package(base_dir: &Path, new_version: &Version) -> Result<()> {
    let mut content = String::new();

    let cargo_toml_file = base_dir.join("Cargo.toml");
    fs::File::open(&cargo_toml_file)?.read_to_string(&mut content)?;

    if !REPLACE_LOCO_LIB_VERSION_.is_match(&content) {
        return Err(Error::BumpVersion {
            path: cargo_toml_file,
            package: "root_package".to_string(),
        });
    }

    let content = REPLACE_LOCO_LIB_VERSION_.replace(&content, |captures: &regex::Captures| {
        format!("{}{}", &captures["name"], new_version)
    });

    let mut modified_file = fs::File::create(cargo_toml_file)?;
    modified_file.write_all(content.as_bytes())?;

    Ok(())
}

/// Update the dependencies of loco-rs in all starter projects to the given version.
///
/// # Errors
/// Returns an error when it could not parse a loco Cargo.toml file or has an error updating the file.
pub fn update_starters(
    base_dir: &Path,
    replace_with: &str,
    replace_migrator: Option<&str>,
) -> Result<()> {
    let starter_projects = utils::get_cargo_folders(&base_dir.join(utils::FOLDER_STARTERS))?;

    for starter_project in starter_projects {
        replace_loco_rs_version(&starter_project, replace_with)?;

        let migration_lock_file = starter_project.join("migration");
        if migration_lock_file.exists() {
            replace_loco_rs_version(
                &migration_lock_file,
                replace_migrator.unwrap_or(replace_with),
            )?;
        }
    }

    Ok(())
}

fn replace_loco_rs_version(path: &Path, replace_with: &str) -> Result<()> {
    let mut content = String::new();
    let cargo_toml_file = path.join("Cargo.toml");
    fs::File::open(&cargo_toml_file)?.read_to_string(&mut content)?;

    if !REPLACE_LOCO_PACKAGE_VERSION.is_match(&content) {
        return Err(Error::BumpVersion {
            path: cargo_toml_file,
            package: "loco-rs".to_string(),
        });
    }
    content = REPLACE_LOCO_PACKAGE_VERSION
        .replace_all(&content, |_captures: &regex::Captures| {
            replace_with.to_string()
        })
        .to_string();

    let mut modified_file = fs::File::create(cargo_toml_file)?;
    modified_file.write_all(content.as_bytes())?;
    Ok(())
}
