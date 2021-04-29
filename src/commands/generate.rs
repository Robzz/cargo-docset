//! Implementation of the `generate` command.

use crate::{
    common::*,
    error::*
};

use rusqlite::{params, Connection};
use snafu::ResultExt;
use toml::Value;

use std::{
    borrow::ToOwned,
    ffi::OsStr,
    fs::{copy, create_dir_all, read_dir, remove_dir_all, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
    str::FromStr
};

#[derive(Debug)]
pub struct GenerateConfig {
    pub manifest_path: Option<String>,
    pub package: Package,
    pub no_dependencies: bool,
    pub doc_private_items: bool,
    pub features: Vec<String>,
    pub no_default_features: bool,
    pub all_features: bool,
    pub target: Option<String>,
    pub exclude: Vec<String>,
    pub clean: bool,
    pub lib: bool,
    pub bin: Vec<String>,
    pub bins: bool
}

impl Default for GenerateConfig {
    fn default() -> GenerateConfig {
        GenerateConfig {
            manifest_path: None,
            package: Package::Current,
            no_dependencies: false,
            doc_private_items: false,
            exclude: Vec::new(),
            features: Vec::new(),
            no_default_features: false,
            all_features: false,
            target: None,
            clean: true,
            lib: false,
            bin: Vec::new(),
            bins: false
        }
    }
}

impl GenerateConfig {
    fn into_args(self) -> Vec<String> {
        let mut args = Vec::new();
        match self.package {
            Package::Current => {}
            Package::All => {
                args.push("--workspace".to_owned());
            }
            Package::Single(package_name) => {
                args.extend_from_slice(&["--package".to_owned(), package_name]);
            }
            Package::List(packages) => {
                for package in packages {
                    args.extend_from_slice(&["--package".to_owned(), package]);
                }
            }
        }
        if self.no_dependencies {
            args.push("--no-deps".to_owned())
        }
        if self.doc_private_items {
            args.push("--document-private-items".to_owned())
        }
        if !self.exclude.is_empty() {
            args.push("--exclude".to_owned());
            args.extend(self.exclude);
        }
        if !self.features.is_empty() {
            args.push("--features".to_owned());
            args.extend(self.features);
        }
        if self.no_default_features {
            args.push("--no-default-features".to_owned())
        }
        if self.all_features {
            args.push("--all-features".to_owned())
        }
        if let Some(target) = self.target {
            args.push("--target".to_owned());
            args.push(target);
        }
        if self.lib {
            args.push("--lib".to_owned());
        }
        if self.bins {
            args.push("bins".to_owned());
        }
        args
    }
}

fn parse_docset_entry<P1: AsRef<Path>, P2: AsRef<Path>>(
    module_path: &Option<&str>,
    rustdoc_root_dir: P1,
    file_path: P2
) -> Option<DocsetEntry> {
    if file_path.as_ref().extension() == Some(OsStr::new("html")) {
        let file_name = file_path.as_ref().file_name().unwrap().to_string_lossy();
        let parts = file_name.split('.').collect::<Vec<_>>();

        let file_db_path = file_path
            .as_ref()
            .strip_prefix(&rustdoc_root_dir)
            .unwrap()
            .to_owned();
        match parts.len() {
            2 => {
                match parts[0] {
                    "index" => {
                        if let Some(mod_path) = module_path {
                            if mod_path.contains(':') {
                                // Module entry
                                Some(DocsetEntry::new(
                                    format!("{}::{}", mod_path, parts[0]),
                                    EntryType::Module,
                                    file_db_path
                                ))
                            } else {
                                // Package entry
                                Some(DocsetEntry::new(
                                    (*mod_path).to_string(),
                                    EntryType::Package,
                                    file_db_path
                                ))
                            }
                        } else {
                            None
                        }
                    }
                    _ => None
                }
            }
            3 => match parts[0] {
                "const" => Some(DocsetEntry::new(
                    format!("{}::{}", module_path.unwrap().to_string(), parts[1]),
                    EntryType::Constant,
                    file_db_path
                )),
                "enum" => Some(DocsetEntry::new(
                    format!("{}::{}", module_path.unwrap().to_string(), parts[1]),
                    EntryType::Enum,
                    file_db_path
                )),
                "fn" => Some(DocsetEntry::new(
                    format!("{}::{}", module_path.unwrap().to_string(), parts[1]),
                    EntryType::Function,
                    file_db_path
                )),
                "macro" => Some(DocsetEntry::new(
                    format!("{}::{}", module_path.unwrap().to_string(), parts[1]),
                    EntryType::Macro,
                    file_db_path
                )),
                "trait" => Some(DocsetEntry::new(
                    format!("{}::{}", module_path.unwrap().to_string(), parts[1]),
                    EntryType::Trait,
                    file_db_path
                )),
                "struct" => Some(DocsetEntry::new(
                    format!("{}::{}", module_path.unwrap().to_string(), parts[1]),
                    EntryType::Struct,
                    file_db_path
                )),
                "type" => Some(DocsetEntry::new(
                    format!("{}::{}", module_path.unwrap().to_string(), parts[1]),
                    EntryType::Type,
                    file_db_path
                )),
                _ => None
            },
            _ => None
        }
    } else {
        None
    }
}

const ROOT_SKIP_DIRS: &[&str] = &["src", "implementors"];

fn recursive_walk(
    root_dir: &Path,
    cur_dir: &Path,
    module_path: Option<&str>
) -> Result<Vec<DocsetEntry>> {
    let dir = read_dir(cur_dir).context(IoRead)?;
    let mut entries = vec![];
    let mut subdir_entries = vec![];

    for dir_entry in dir {
        let dir_entry = dir_entry.unwrap();
        if dir_entry.file_type().unwrap().is_dir() {
            let mut subdir_module_path =
                module_path.map(|p| format!("{}::", p)).unwrap_or_default();
            let dir_name = dir_entry.file_name().to_string_lossy().to_string();

            // Ignore some of the root directories which are of no interest to us
            if !(module_path.is_none() && ROOT_SKIP_DIRS.contains(&dir_name.as_str())) {
                subdir_module_path.push_str(&dir_name);
                subdir_entries.push(recursive_walk(
                    &root_dir,
                    &dir_entry.path(),
                    Some(&subdir_module_path)
                ));
            }
        } else if let Some(entry) = parse_docset_entry(&module_path, &root_dir, &dir_entry.path()) {
            entries.push(entry);
        }
    }
    for v in subdir_entries {
        entries.extend(v?);
    }
    Ok(entries)
}

fn generate_sqlite_index<P: AsRef<Path>>(docset_dir: P, entries: Vec<DocsetEntry>) -> Result<()> {
    let mut conn_path = docset_dir.as_ref().to_owned();
    conn_path.push("Contents");
    conn_path.push("Resources");
    conn_path.push("docSet.dsidx");
    let mut conn = Connection::open(&conn_path).context(Sqlite)?;
    conn.execute(
        "CREATE TABLE searchIndex(id INTEGER PRIMARY KEY, name TEXT, type TEXT, path TEXT);
        CREATE UNIQUE INDEX anchor ON searchIndex (name, type, path);
        )",
        params![]
    )
    .context(Sqlite)?;
    let transaction = conn.transaction().context(Sqlite)?;
    {
        let mut stmt = transaction
            .prepare("INSERT INTO searchIndex (name, type, path) VALUES (?1, ?2, ?3)")
            .context(Sqlite)?;
        for entry in entries {
            stmt.execute([
                entry.name,
                entry.ty.to_string(),
                entry.path.to_str().unwrap().to_owned()
            ])
            .context(Sqlite)?;
        }
    }
    transaction.commit().context(Sqlite)?;
    Ok(())
}

fn copy_dir_recursive<Ps: AsRef<Path>, Pd: AsRef<Path>>(src: Ps, dst: Pd) -> Result<()> {
    create_dir_all(&dst).context(IoWrite)?;
    for entry in read_dir(&src).context(IoRead)? {
        let entry = entry.context(IoWrite)?.path();
        if entry.is_dir() {
            let mut dst_dir = dst.as_ref().to_owned();
            dst_dir.push(entry.strip_prefix(&src).unwrap());
            copy_dir_recursive(entry, dst_dir)?;
        } else if entry.is_file() {
            let mut dst_file = dst.as_ref().to_owned();
            dst_file.push(entry.file_name().unwrap());
            copy(entry, dst_file).context(IoWrite)?;
        }
    }
    Ok(())
}

fn write_metadata<P: AsRef<Path>>(docset_root_dir: P, package_name: &str, is_virtual_manifest: bool, first_package_name: Option<&str>) -> Result<()> {
    let mut info_plist_path = docset_root_dir.as_ref().to_owned();
    info_plist_path.push("Contents");
    info_plist_path.push("Info.plist");

    let mut info_file = File::create(info_plist_path).context(IoWrite)?;
    let docset_index = if is_virtual_manifest {
        format!("{}/index.html", first_package_name.unwrap())
    } else {
        format!("{}/index.html", package_name)
    };
    write!(info_file,
        "\
        <?xml version=\"1.0\" encoding=\"UTF-8\"?>
        <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
        <plist version=\"1.0\">
        <dict>
            <key>CFBundleIdentifier</key>
                <string>{}</string>
            <key>CFBundleName</key>
                <string>{}</string>
            <key>dashIndexFilePath</key>
                <string>{}</string>
            <key>DocSetPlatformFamily</key>
                <string>{}</string>
            <key>isDashDocset</key>
                <true/>
            <key>isJavaScriptEnabled</key>
                <true/>
        </dict>
        </plist>",
         package_name, package_name, docset_index, package_name).context(IoWrite)?;
    Ok(())
}

pub fn generate(cfg: GenerateConfig) -> Result<()> {
    // Step 1: generate rustdoc
    // Figure out for which crate to build the doc and invoke cargo doc.
    // If no crate is specified, run cargo doc for the current crate/workspace.
    if cfg.package != Package::All && !cfg.exclude.is_empty() {
        return Args {
            msg: "--exclude must be used with --all"
        }
        .fail();
    }
    // Determine the package name that we will use for the generated docset.
    // If a single package was requested, use this one.
    // Otherwise, we use the "root" package/workspace name.
    // TODO: we should probably handle the Package::List case differently. Maybe provide an option
    // to set the generated docset name ?
    let mut is_virtual_manifest = false;
    let mut first_package_name = None;
    let package_name = match cfg.package.clone() {
        Package::Single(package) => package,
        _ => {
            let manifest_location =
                cfg.manifest_path
                    .clone()
                    .unwrap_or(locate_package_manifest()?);
            let mut manifest_file = File::open(&manifest_location)
                .context(IoRead)?;
            let mut manifest_contents = String::new();
            manifest_file
                .read_to_string(&mut manifest_contents)
                .context(IoRead)?;
            let toml_manifest = manifest_contents.parse::<Value>().context(Toml)?;
            let package_table = toml_manifest
                .as_table()
                .expect("Cargo.toml is not a toml table")
                .get("package");
            match package_table {
                Some(toml) => {
                    toml.as_table()
                    .expect("Cargo.toml package entry is not a toml table")
                    .get("name")
                    .expect("Cargo.toml doesn't define a package name")
                    .as_str()
                    .expect("Cargo.toml package.name entry is not a string")
                    .to_owned()
                }
                None => {
                    is_virtual_manifest = true;
                    let members = toml_manifest.as_table()
                        .unwrap()
                        .get("workspace")
                        .expect("Manifest has neither a package nor a workspace section")
                        .as_table()
                        .expect("Workspace section is not a toml table")
                        .get("members")
                        .expect("Workspace section does not have a member field")
                        .as_array()
                        .expect("Members field is not an array");
                    first_package_name = Some(
                        members
                            .first()
                            .expect("Virtual manifest workspace has no members")
                            .as_str()
                            .expect("Workspace member is not a string")
                            .to_owned()
                    );
                    let manifest_path = Path::new(&manifest_location);
                    manifest_path
                        .parent()
                        .expect("Manifest parent location is not a directory ???")
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string()
                }
            }
        }
    };
    let cargo_metadata = get_cargo_metadata()?;
    let mut docset_root_dir = PathBuf::from_str(&cargo_metadata.target_directory).unwrap();
    let mut rustdoc_root_dir = docset_root_dir.clone();
    rustdoc_root_dir.push("doc");
    docset_root_dir.push("docset");
    docset_root_dir.push(format!("{}.docset", package_name));

    // Clean the documentation directory if so requested
    if cfg.clean {
        println!("Running 'cargo clean --doc'...");
        let mut cargo_clean_args = vec!["clean".to_owned()];
        if let Some(ref manifest_path) = &cfg.manifest_path {
            cargo_clean_args.push("--manifest-path".to_owned());
            cargo_clean_args.push(manifest_path.to_owned());
        }
        let cargo_clean_result = Command::new("cargo")
            .args(cargo_clean_args)
            .arg("--doc")
            .status()
            .context(Spawn)?;
        if !cargo_clean_result.success() {
            return CargoClean {
                code: cargo_clean_result.code()
            }
            .fail();
        }
    }
    // Good to go, generate the documentation.
    println!("Running 'cargo doc'...");
    let cargo_doc_result = Command::new("cargo")
        .arg("doc")
        .args(cfg.into_args())
        .status()
        .context(Spawn)?;
    if !cargo_doc_result.success() {
        return CargoDoc {
            code: cargo_doc_result.code()
        }
        .fail();
    }

    // Step 2: iterate over all the html files in the doc directory and parse the filenames
    let entries = recursive_walk(&rustdoc_root_dir, &rustdoc_root_dir, None)?;

    // Step 3: generate the SQLite database
    // At this point, we need to start writing into the output docset directory, so create the
    // hirerarchy, and clean it first if it already exists.
    if docset_root_dir.exists() {
        remove_dir_all(&docset_root_dir).context(IoWrite)?;
    }
    let mut docset_hierarchy = docset_root_dir.clone();
    docset_hierarchy.push("Contents");
    docset_hierarchy.push("Resources");
    create_dir_all(&docset_hierarchy).context(IoWrite)?;
    generate_sqlite_index(&docset_root_dir, entries)?;

    // Step 4: Copy the rustdoc to the docset directory
    docset_hierarchy.push("Documents");
    copy_dir_recursive(&rustdoc_root_dir, &docset_hierarchy)?;

    // Step 5: add the required metadata
    write_metadata(&docset_root_dir, &package_name, is_virtual_manifest, first_package_name.as_deref())?;

    println!("Docset succesfully generated in {}", docset_root_dir.to_string_lossy());

    Ok(())
}
