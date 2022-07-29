//! Implementation of the `docset` subcommand.

use crate::{common::*, error::*, io::*, DocsetParams};

use cargo_metadata::Metadata;
use rusqlite::{params, Connection};
use snafu::{ensure, ResultExt};

use std::{
    borrow::ToOwned,
    ffi::OsStr,
    fs::{copy, create_dir_all, read_dir, remove_dir_all, File},
    io::Write,
    path::Path,
    process::Command
};

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
                    format!("{}::{}", module_path.unwrap(), parts[1]),
                    EntryType::Constant,
                    file_db_path
                )),
                "enum" => Some(DocsetEntry::new(
                    format!("{}::{}", module_path.unwrap(), parts[1]),
                    EntryType::Enum,
                    file_db_path
                )),
                "fn" => Some(DocsetEntry::new(
                    format!("{}::{}", module_path.unwrap(), parts[1]),
                    EntryType::Function,
                    file_db_path
                )),
                "macro" => Some(DocsetEntry::new(
                    format!("{}::{}", module_path.unwrap(), parts[1]),
                    EntryType::Macro,
                    file_db_path
                )),
                "trait" => Some(DocsetEntry::new(
                    format!("{}::{}", module_path.unwrap(), parts[1]),
                    EntryType::Trait,
                    file_db_path
                )),
                "struct" => Some(DocsetEntry::new(
                    format!("{}::{}", module_path.unwrap(), parts[1]),
                    EntryType::Struct,
                    file_db_path
                )),
                "type" => Some(DocsetEntry::new(
                    format!("{}::{}", module_path.unwrap(), parts[1]),
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
    let dir = read_dir(cur_dir).context(IoReadSnafu)?;
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
                    root_dir,
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
    let mut conn = Connection::open(&conn_path).context(SqliteSnafu)?;
    conn.execute(
        "CREATE TABLE searchIndex(id INTEGER PRIMARY KEY, name TEXT, type TEXT, path TEXT);
        CREATE UNIQUE INDEX anchor ON searchIndex (name, type, path);
        )",
        params![]
    )
    .context(SqliteSnafu)?;
    let transaction = conn.transaction().context(SqliteSnafu)?;
    {
        let mut stmt = transaction
            .prepare("INSERT INTO searchIndex (name, type, path) VALUES (?1, ?2, ?3)")
            .context(SqliteSnafu)?;
        for entry in entries {
            stmt.execute([
                entry.name,
                entry.ty.to_string(),
                entry.path.to_str().unwrap().to_owned()
            ])
            .context(SqliteSnafu)?;
        }
    }
    transaction.commit().context(SqliteSnafu)?;
    Ok(())
}

fn copy_dir_recursive<Ps: AsRef<Path>, Pd: AsRef<Path>>(src: Ps, dst: Pd) -> Result<()> {
    create_dir_all(&dst).context(IoWriteSnafu)?;
    for entry in read_dir(&src).context(IoReadSnafu)? {
        let entry = entry.context(IoWriteSnafu)?.path();
        if entry.is_dir() {
            let mut dst_dir = dst.as_ref().to_owned();
            dst_dir.push(entry.strip_prefix(&src).unwrap());
            copy_dir_recursive(entry, dst_dir)?;
        } else if entry.is_file() {
            let mut dst_file = dst.as_ref().to_owned();
            dst_file.push(entry.file_name().unwrap());
            copy(entry, dst_file).context(IoWriteSnafu)?;
        }
    }
    Ok(())
}

fn write_metadata<P: AsRef<Path>>(
    docset_root_dir: P,
    docset_name: &str,
    index_package: Option<String>,
    platform_family: Option<String>
) -> Result<()> {
    let mut info_plist_path = docset_root_dir.as_ref().to_owned();
    info_plist_path.push("Contents");
    info_plist_path.push("Info.plist");

    let mut info_file = File::create(info_plist_path).context(IoWriteSnafu)?;
    let index_entry = if let Some(index_package) = index_package {
        format!(
            "<key>dashIndexFilePath</key>
                    <string>{}/index.html</string>",
            index_package
        )
    } else {
        String::new()
    };
    let identifier_entry = if let Some(platform_family) = &platform_family {
        format!(
            "<key>CFBundleIdentifier</key>
                    <string>{}</string>",
            platform_family
        )
    } else {
        String::new()
    };

    let platform_family_entry = if let Some(platform_family) = &platform_family {
        format!(
            "<key>DocSetPlatformFamily</key>
                    <string>{}</string>",
            platform_family
        )
    } else {
        String::new()
    };

    write!(info_file,
        "\
        <?xml version=\"1.0\" encoding=\"UTF-8\"?>
        <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
        <plist version=\"1.0\">
        <dict>
            {}
            <key>CFBundleName</key>
                <string>{}</string>
            {}
            {}
            <key>isDashDocset</key>
                <true/>
            <key>isJavaScriptEnabled</key>
                <true/>
        </dict>
        </plist>",
        identifier_entry, docset_name, index_entry, platform_family_entry).context(IoWriteSnafu)?;
    Ok(())
}

/// Determine the name we will use for the generated docset.
/// If a name was provided on the command line, we use this one.
/// If no name was provided:
///   * If a single package was requested, use this one.
///   * Otherwise, if there is a workspace root package and we have been asked to generate
///     documentation for it, use this one.
///   * Otherwise, generate a name from the list of workspace member packages being built.
fn get_docset_name(cfg: &DocsetParams, metadata: &Metadata) -> String {
    if let Some(docset_name) = &cfg.docset_name {
        return docset_name.to_owned();
    }

    match (cfg.workspace.all, cfg.workspace.package.len()) {
        (false, 1) => cfg.workspace.package[0].to_owned(),
        _ => {
            if let Some(root_package) = metadata.root_package() {
                root_package.name.to_owned()
            } else {
                let package_list = cfg.workspace.package.join(", ");
                format!("Docset for packages {}", package_list)
            }
        }
    }
}

/// Return the name of the package that should be used for the docset index, if any.
/// This uses the same rules as docset name selection, except no index is a valid option.
fn get_docset_index(cfg: &DocsetParams, metadata: &Metadata) -> Option<String> {
    if cfg.docset_index.is_some() {
        return cfg.docset_index.clone();
    }

    match (cfg.workspace.all, cfg.workspace.package.len()) {
        (false, 1) => Some(cfg.workspace.package[0].to_owned()),
        _ => metadata.root_package().map(|p| p.name.to_owned())
    }
}

/// Return the keywork that should be used for the docset platform family, if any.
/// This uses the same rules as docset name selection, except no identifier is a valid option.
fn get_docset_platform_family(cfg: &DocsetParams, metadata: &Metadata) -> Option<String> {
    if let Some(platform_family) = &cfg.platform_family {
        return Some(platform_family.to_owned());
    }

    match (cfg.workspace.all, cfg.workspace.package.len()) {
        (false, 1) => Some(cfg.workspace.package[0].to_owned()),
        _ => metadata.root_package().map(|p| p.name.to_owned())
    }
}

pub fn generate_docset(cfg: DocsetParams) -> Result<()> {
    // Step 1: generate rustdoc
    // Figure out for which crate to build the doc and invoke cargo doc.
    // If no crate is specified, run cargo doc for the current crate/workspace.
    if cfg.workspace.all {
        ensure!(
            cfg.workspace.exclude.is_empty(),
            ArgsSnafu {
                msg: "--exclude must be used with --all"
            }
        );
    }

    let cargo_metadata = cfg.manifest.metadata().exec().context(CargoMetadataSnafu)?;

    // Clean the documentation directory if the user didn't explicitly ask not to clean it.
    if !cfg.no_clean {
        println!("Running 'cargo clean --doc'...");
        let mut cargo_clean_args = vec!["clean".to_owned()];
        if let Some(ref manifest_path) = &cfg.manifest.manifest_path {
            cargo_clean_args.push("--manifest-path".to_owned());
            cargo_clean_args.push(manifest_path.to_string_lossy().to_string());
        }
        let cargo_clean_result = Command::new("cargo")
            .args(cargo_clean_args)
            .arg("--doc")
            .status()
            .context(SpawnSnafu)?;
        if !cargo_clean_result.success() {
            return CargoCleanSnafu {
                code: cargo_clean_result.code()
            }
            .fail();
        }
    }
    // Good to go, generate the documentation.
    println!("Running 'cargo doc'...");
    let args = cfg.clone().into_args();
    dbg!(&args);
    let cargo_doc_result = Command::new("cargo")
        .arg("doc")
        .args(args)
        .status()
        .context(SpawnSnafu)?;
    if !cargo_doc_result.success() {
        return CargoDocSnafu {
            code: cargo_doc_result.code()
        }
        .fail();
    }

    // Step 2: iterate over all the html files in the doc directory and parse the filenames
    let docset_name = get_docset_name(&cfg, &cargo_metadata);
    let mut docset_root_dir = cfg
        .target_dir
        .clone()
        .unwrap_or_else(|| cargo_metadata.target_directory.clone().into_std_path_buf());
    let mut rustdoc_root_dir = docset_root_dir.clone();
    rustdoc_root_dir.push("doc");
    docset_root_dir.push("docset");
    let docset_identifier = get_docset_platform_family(&cfg, &cargo_metadata);
    docset_root_dir.push(format!("{}.docset", docset_identifier.clone().unwrap_or("generated-docset.docset".to_owned())));
    let entries = recursive_walk(&rustdoc_root_dir, &rustdoc_root_dir, None)?;

    // Step 3: generate the SQLite database
    // At this point, we need to start writing into the output docset directory, so create the
    // hirerarchy, and clean it first if it already exists.
    if docset_root_dir.exists() {
        remove_dir_all(&docset_root_dir).context(IoWriteSnafu)?;
    }
    let mut docset_hierarchy = docset_root_dir.clone();
    docset_hierarchy.push("Contents");
    docset_hierarchy.push("Resources");
    create_dir_all(&docset_hierarchy).context(IoWriteSnafu)?;
    generate_sqlite_index(&docset_root_dir, entries)?;

    // Step 4: Copy the rustdoc to the docset directory
    docset_hierarchy.push("Documents");
    copy_dir_recursive(&rustdoc_root_dir, &docset_hierarchy)?;

    // Step 5: add the required metadata
    if docset_identifier.is_none() {
        warn("no docset identifier was provided and none could be generated, consider adding the '--docset-identifier' option.");
    }

    write_metadata(
        &docset_root_dir,
        &docset_name,
        get_docset_index(&cfg, &cargo_metadata),
        docset_identifier
    )?;

    println!(
        "Docset succesfully generated in {}",
        docset_root_dir.to_string_lossy()
    );

    Ok(())
}
