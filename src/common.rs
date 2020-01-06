use derive_more::Constructor;

use std::{fmt::Display, path::PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Package {
    All,
    Current,
    Single(String),
    List(Vec<String>),
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
    Type, //Union // Is this even implemented in Rust ?
}

impl Display for EntryType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            EntryType::Constant => write!(f, "Constant"),
            EntryType::Enum => write!(f, "Enum"),
            EntryType::Function => write!(f, "Function"),
            EntryType::Macro => write!(f, "Macro"),
            EntryType::Module => write!(f, "Module"),
            EntryType::Package => write!(f, "Package"),
            EntryType::Struct => write!(f, "Struct"),
            EntryType::Trait => write!(f, "Trait"),
            EntryType::Type => write!(f, "Type"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Constructor)]
pub struct DocsetEntry {
    pub name: String,
    pub ty: EntryType,
    pub path: PathBuf,
}
