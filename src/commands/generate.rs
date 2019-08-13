//! Implementation of the `generate` command.

use crate::common::{DocsetEntry, EntryType, Package};

use cargo::{
    Config as CargoConfig,
    core::{
        compiler::CompileMode,
        Workspace
    },
    ops::{doc, CompileOptions, DocOptions, Packages}
};

use std::{
    borrow::{Borrow, ToOwned},
    ffi::OsStr,
    fs::read_dir,
    path::{Path, PathBuf}
};

pub struct GenerateConfig {
    pub package: Package,
    pub dependencies: bool
}

impl GenerateConfig {
    /// Create a new GenerateConfig.
    pub fn new(package: Package, dependencies: bool) -> GenerateConfig {
        GenerateConfig { package, dependencies }
    }
}

impl Default for GenerateConfig {
    fn default() -> GenerateConfig {
        GenerateConfig { package: Package::Current, dependencies: false }
    }
}

fn parse_docset_entry<P: AsRef<Path> + std::fmt::Debug>(module_path: &Option<&str>, file_path: P) -> Option<DocsetEntry> {
    if file_path.as_ref().extension() == Some(OsStr::new("html")) {
        let file_name = file_path.as_ref().file_name().unwrap().to_string_lossy();
        println!("Parsing file {:?}::{}", module_path, file_name);
        let parts = file_name.split(".").collect::<Vec<_>>();
        match parts.len() {
            2 => {
                match parts[0] {
                    "index" => {
                        if let Some(mod_path) = module_path {
                            if mod_path.contains(':') {
                                // Module entry
                                Some(DocsetEntry::new(format!("{}::{}", mod_path, parts[0]), EntryType::Module, file_path.as_ref().to_owned()))
                            }
                            else {
                                // Package entry
                                Some(DocsetEntry::new(mod_path.to_string(), EntryType::Package, file_path.as_ref().to_owned()))
                            }
                        }
                        else { None }
                    }
                    _ => None
                }
            }
            _ => None
        }
    }
    else {
        None
    }
}

const ROOT_SKIP_DIRS: &[&'static str] = &["src", "implementors"];

fn recursive_walk<P: AsRef<Path> + std::fmt::Debug>(root_dir: P, module_path: Option<&str>) -> Vec<DocsetEntry> {
    let dir = read_dir(root_dir.as_ref()).unwrap();
    let mut entries = vec![];
    let mut subdir_entries = vec![];

    println!("Scanning directory {:?}, (module path: {:?})", root_dir, module_path);

    for dir_entry in dir {
        let dir_entry = dir_entry.unwrap();
        if dir_entry.file_type().unwrap().is_dir() {
            let mut subdir_module_path = module_path.map(|p| format!("{}::", p)).unwrap_or(String::new());
            let dir_name = dir_entry.file_name().to_string_lossy().to_string();

            // Ignore some of the root directories which are of no interest to us
            if !(module_path.is_none() && ROOT_SKIP_DIRS.contains(&dir_name.as_str())) {
                subdir_module_path.push_str(&dir_name);
                subdir_entries.push(recursive_walk(dir_entry.path(), Some(&subdir_module_path)));
            }
        }
        else {
            if let Some(entry) = parse_docset_entry(&module_path, &dir_entry.path()) {
                entries.push(entry);
            }
        }
    }
    for v in subdir_entries {
        entries.extend(v);
    }
    entries
}

pub fn generate(cargo_cfg: &CargoConfig, workspace: &Workspace, cfg: GenerateConfig) {
    // Step 1: generate rustdoc
    // Figure out for which crate to build the doc and invoke cargo doc.
    // If no crate is specified, run cargo doc for the current crate/workspace.
    // Other options:
    // * --all
    // * -p, --package
    // * -d, --dependencies
    let mut compile_opts = CompileOptions::new(&cargo_cfg, CompileMode::Doc { deps: cfg.dependencies }).unwrap();
    let cur_package = workspace.current();
    let root_package_name = match cfg.package {
        Package::All => { compile_opts.spec = Packages::All; cur_package.unwrap().name().as_str().to_owned() }
        Package::Current => { compile_opts.spec = Packages::Default; cur_package.unwrap().name().as_str().to_owned() }
        Package::Single(name) => { compile_opts.spec = Packages::Packages(vec![name.clone()]); name }
    };

    let doc_cfg = DocOptions { open_result: false, compile_opts };
    doc(&workspace, &doc_cfg).unwrap();
    println!("Generated rustdoc for package {}", root_package_name);

    // Step 2: iterate over all the html files in the doc directory and parse the filenames
    let mut root_doc_dir = PathBuf::new();
    root_doc_dir.push(workspace.root());
    root_doc_dir.push("target");
    root_doc_dir.push("doc");
    let entries = recursive_walk(root_doc_dir, None);

    println!("Got the following entries: {:?}", entries);

    // Step 3: generate the SQLite database
}
