use clap::{Arg, App, SubCommand};
use cargo::{
    Config as CargoConfig,
    core::Workspace,
    util::important_paths::find_root_manifest_for_wd
};

use std::env::current_dir;

enum Error {
}

mod commands;
mod common;

use common::Package;
use commands::generate::{generate, GenerateConfig};

// TODO: generate the docs in a different target directory that we can claim as our own

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
            .about("Generate a Dash/Zeal compatible docset for the specified package.")
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

    let mut cargo_cfg = CargoConfig::default().unwrap();
    cargo_cfg.configure(0, Some(false), &None, false, false, false, &None, &[]).unwrap();

    match matches.subcommand_name() {
        Some("generate") | None => {
            let cfg = match matches.subcommand_matches("generate") {
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

            generate(&cargo_cfg, &workspace, cfg);
        }
        _ => {
            println!("Unknown command.")
        }
    }
}
