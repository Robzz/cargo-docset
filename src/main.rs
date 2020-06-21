use clap::{crate_authors, crate_version, App, Arg, ArgMatches, SubCommand};

mod commands;
mod common;
mod error;

use crate::error::*;
use commands::generate::{generate, GenerateConfig};
use common::Package;

use std::process::exit;

fn run(sub_matches: &ArgMatches) -> Result<()> {
    let quiet = sub_matches.is_present("quiet");
    let verbosity_level = sub_matches.occurrences_of("verbose") as u32;

    if quiet && verbosity_level != 0 {
        eprintln!("Error: cannot specify `--quiet` with `--verbose`.");
        exit(1);
    }

    let mut cfg = GenerateConfig::default();
    cfg.manifest_path = sub_matches.value_of("manifest-path").map(String::from);
    cfg.no_dependencies = sub_matches.is_present("no-deps");
    cfg.package = if sub_matches.is_present("all") {
        Package::All
    } else if let Some(packages) = sub_matches.values_of_lossy("package") {
        if packages.len() == 1 {
            Package::Single(packages[0].clone())
        } else {
            Package::List(packages)
        }
    } else {
        Package::Current
    };
    cfg.doc_private_items = sub_matches.is_present("document-private-items");
    cfg.exclude = sub_matches
        .values_of_lossy("exclude")
        .unwrap_or_else(Vec::new);
    cfg.all_features = sub_matches.is_present("all-features");
    cfg.no_default_features = sub_matches.is_present("no-default-features");
    if sub_matches.is_present("features") {
        cfg.features = sub_matches.values_of_lossy("features").unwrap();
    }
    cfg.target = sub_matches.value_of("target").map(String::from);
    cfg.clean = !sub_matches.is_present("no-clean");
    cfg.lib = sub_matches.is_present("lib");
    cfg.bins = sub_matches.is_present("bins");
    if sub_matches.is_present("bin") {
        cfg.bin = sub_matches.values_of_lossy("bin").unwrap_or_else(Vec::new);
    }

    generate(cfg)
}

fn main() {
    let matches = App::new("cargo-docset")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Generates a Zeal/Dash docset from a crate documentation.")
        .bin_name("cargo")
        .subcommand(
            SubCommand::with_name("docset")
                .about("Generates a docset")
                .arg(
                    Arg::from_usage("-p, --package <SPEC>...  'Package(s) to document'")
                        .required(false)
                        .takes_value(true)
                        .multiple(true)
                )
                .arg(
                    Arg::from_usage(
                        "--exclude <SPEC>...  'Package(s) to exclude from the documentation'"
                    )
                    .multiple(true)
                    .required(false)
                )
                .arg(
                    Arg::from_usage(
                        "-v, --verbose  'Enable verbose output (-vv for extra verbosity)'"
                    )
                    .multiple(true)
                )
                .arg(
                    Arg::from_usage(
                        "--bin <BIN> 'Document only the specified binary'"
                    )
                    .multiple(true)
                    .required(false)
                )
                .arg(
                    Arg::from_usage("--features <FEATURES> 'Space separated list of features to activate'")
                        .multiple(true)
                        .required(false)
                )
                .arg(
                    Arg::from_usage("--target <TRIPLE> 'Build for the specified target triple'")
                        .required(false)
                )
                .arg(
                    Arg::from_usage("--manifest-path <PATH> 'Path to Cargo.toml")
                        .required(false)
                )
                .args_from_usage(
                    "-q, --quiet             'Suppress all output to stdout.'
                    -C, --no-clean           'Do not clean the doc directory before generating the rustdoc'
                    --all                    'Document all packages in the workspace'
                    --lib                    'Document only this package's library'
                    --bins                   'Document all binaries'
                    --no-deps                'Don't build documentation for dependencies'
                    --document-private-items 'Document private items'
                    --all-features           'Build with all features enabled'
                    --no-default-features    'Build without the 'default' feature'
                    --frozen                 'Require Cargo.lock and cache are up to date'
                    --locked                 'Require Cargo.lock is up to date'
                    --offline                'Run without accessing the network'"
                )
        )
        .get_matches();
    if let Some(sub_matches) = matches.subcommand_matches("docset") {
        if let Err(e) = run(sub_matches) {
            eprintln!("{}", e);
            exit(1);
        }
    } else {
        println!("Invalid arguments.");
        println!("{}", matches.usage());
        exit(1);
    }
}
