use clap::{Arg, App, SubCommand};
use cargo::{
    Config,
    core::{
        compiler::CompileMode,
        Workspace
    },
    ops::{doc, CompileOptions, DocOptions, Packages},
    util::important_paths::find_root_manifest_for_wd
};

use std::env::current_dir;

enum Error {
}

enum Package {
    All,
    Current,
    Single(String)
}

struct GenerateConfig {
    pub package: Package,
    pub dependencies: bool
}

impl GenerateConfig {
    /// Create a new GenerateConfig.
    fn new(package: Package, dependencies: bool) -> GenerateConfig {
        GenerateConfig { package, dependencies }
    }
}

impl Default for GenerateConfig {
    fn default() -> GenerateConfig {
        GenerateConfig { package: Package::Current, dependencies: false }
    }
}

fn main() {
    let matches =
        App::new("cargo-docset")
        .version("0.1")
        .author("Robin Chavignat")
        .about("Generates a Zeal/Dash docset from a crate documentation.")
        .arg(Arg::with_name("config")
            .help("Sets a custom config file")
            .takes_value(true))
        .arg(Arg::with_name("v")
           .short("v")
           .multiple(true)
           .help("Sets the level of verbosity"))
        .subcommand(
            SubCommand::with_name("generate")
            .about("Generate a Dash/Zeal compatible docset for the specified crate.")
            .arg(
                Arg::with_name("package")
                .short("p")
                .takes_value(true)
                .help("Package to document, as understood by `cargo doc`.")
            )
            .arg(
                Arg::with_name("dependencies")
                .short("d")
                .takes_value(false)
                .help("Enable documenting the package dependencies.")
        ))
        .get_matches();

    let mut cargo_cfg = Config::default().unwrap();
    cargo_cfg.configure(0, Some(false), &None, false, false, false, &None, &[]).unwrap();

    match matches.subcommand_name() {
        Some("generate") | None => {
            let config = match matches.subcommand_matches("generate") {
                Some(sub_matches) => {
                    let package = sub_matches.value_of("package").map(|p| Package::Single(p.to_owned())).unwrap_or(Package::Current);
                    let dependencies = sub_matches.is_present("dependencies");
                    GenerateConfig::new(package, dependencies)
                },
                None => GenerateConfig::default()
            };
            let cur_dir = current_dir().unwrap();
            let root_manifest = find_root_manifest_for_wd(&cur_dir).unwrap();
            let workspace = Workspace::new(&root_manifest, &cargo_cfg).unwrap();

            // Step 1: generate rustdoc
            // Figure out for which crate to build the doc and invoke cargo doc.
            // If no crate is specified, run cargo doc for the current crate/workspace.
            // Other options:
            // * --all
            // * -p, --package
            // * -d, --dependencies
            let mut compile_opts = CompileOptions::new(&cargo_cfg, CompileMode::Doc { deps: config.dependencies }).unwrap();
            let cur_package = workspace.current();
            let root_package_name = match config.package {
                Package::All => { compile_opts.spec = Packages::All; cur_package.unwrap().name().as_str().to_owned() }
                Package::Current => { compile_opts.spec = Packages::Default; cur_package.unwrap().name().as_str().to_owned() }
                Package::Single(name) => { compile_opts.spec = Packages::Packages(vec![name.clone()]); name }
            };

            let doc_cfg = DocOptions { open_result: false,
                compile_opts
            };
            doc(&workspace, &doc_cfg).unwrap();
            println!("Generated rustdoc for package {}", root_package_name);

            // Step 2: iterate over all the html files in the doc directory and parse the filenames
        }
        _ => {
            println!("Unknown command.")
        }
    }
}
