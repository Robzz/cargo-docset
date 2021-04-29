use crate::error::*;

use derive_more::Constructor;
use serde::Deserialize;
use snafu::ResultExt;

use std::{
    fmt::Display,
    path::PathBuf,
    process::Command,
    result::Result as StdResult,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Package {
    All,
    Current,
    Single(String),
    List(Vec<String>)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryType {
    Constant,
    Enum,
    Function,
    Macro,
    Module,
    Package, // i.e. crate
    Struct,
    Trait,
    Type //Union // Is this even implemented in Rust ?
}

impl Display for EntryType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> StdResult<(), std::fmt::Error> {
        match self {
            EntryType::Constant => write!(f, "Constant"),
            EntryType::Enum => write!(f, "Enum"),
            EntryType::Function => write!(f, "Function"),
            EntryType::Macro => write!(f, "Macro"),
            EntryType::Module => write!(f, "Module"),
            EntryType::Package => write!(f, "Package"),
            EntryType::Struct => write!(f, "Struct"),
            EntryType::Trait => write!(f, "Trait"),
            EntryType::Type => write!(f, "Type")
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Constructor)]
pub struct DocsetEntry {
    pub name: String,
    pub ty: EntryType,
    pub path: PathBuf
}

#[derive(Debug, Deserialize)]
struct ManifestLocation {
    pub root: String,
}

#[derive(Debug, Deserialize)]
pub struct CargoMetadata {
    pub workspace_root: String,
    pub target_directory: String,
}

pub fn locate_package_manifest() -> Result<String> {
    // Use the cargo `locate-project` subcommand to locate the package manifest.
    let cargo_locate_result = Command::new("cargo")
        .args(vec!["locate-project"])
        .output()
        .context(Spawn)?;
    let dir: ManifestLocation = serde_json::from_slice(&cargo_locate_result.stdout).context(Json)?;
    Ok(dir.root)
}

pub fn get_cargo_metadata() -> Result<CargoMetadata> {
    // Use the cargo `metadata` subcommand to locate the workspace root and other useful information.
    let cargo_locate_result = Command::new("cargo")
        .args(vec!["metadata", "--no-deps", "--format-version", "1"])
        .output()
        .context(Spawn)?;
    serde_json::from_slice::<CargoMetadata>(&cargo_locate_result.stdout).context(Json)
}
