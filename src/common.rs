use derive_more::Constructor;

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Package {
    All,
    Current,
    Single(String)
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
    Type,
    //Union // Is this even implemented in Rust ?
}

#[derive(Debug, Clone, PartialEq, Eq, Constructor)]
pub struct DocsetEntry {
    pub name: String,
    pub ty: EntryType,
    pub path: PathBuf
}
