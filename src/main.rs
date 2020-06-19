use cargo::{
    core::Workspace, util::important_paths::find_root_manifest_for_wd, Config as CargoCfg
};
use clap::{crate_authors, crate_version, App, Arg, ArgMatches, SubCommand};
use snafu::ResultExt;

use std::env::current_dir;

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

    let mut cargo_cfg = CargoCfg::default().context(CargoConfig)?;
    cargo_cfg
        .configure(
            verbosity_level,
            Some(quiet),
            &None,
            sub_matches.is_present("frozen"),
            sub_matches.is_present("locked"),
            sub_matches.is_present("offline"),
            &None,
            &[]
        )
        .context(CargoConfig)?;

    let mut cfg = GenerateConfig::default();
    cfg.no_dependencies = sub_matches.is_present("no-deps");
    cfg.package = if sub_matches.is_present("all") {
        Package::All
    } else if let Some(packages) = sub_matches.values_of_lossy("package") {
        Package::List(packages)
    } else if let Some(package) = sub_matches.value_of("package") {
        Package::Single(package.to_owned())
    } else {
        Package::Current
    };
    cfg.doc_private_items = sub_matches.is_present("document-private-items");
    cfg.exclude = sub_matches
        .values_of_lossy("exclude")
        .unwrap_or_else(Vec::new);
    if sub_matches.is_present("all-features") {
        cfg.all_features = true;
    }
    if sub_matches.is_present("no-default-features") {
        cfg.no_default_features = true;
    }
    if sub_matches.is_present("features") {
        cfg.features = sub_matches.values_of_lossy("features").unwrap();
    }
    if sub_matches.is_present("no-clean") {
        cfg.clean = false;
    }
    if sub_matches.is_present("lib") {
        cfg.lib = true;
    }
    if sub_matches.is_present("bins") {
        cfg.bins = Some(vec![])
    } else if sub_matches.is_present("bin") {
        cfg.bins = sub_matches.values_of_lossy("bin");
    }

    let cur_dir = current_dir().context(Cwd)?;
    let root_manifest = find_root_manifest_for_wd(&cur_dir).context(CargoConfig)?;
    let workspace = Workspace::new(&root_manifest, &cargo_cfg).context(CargoConfig)?;

    generate(&cargo_cfg, &workspace, cfg)
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
