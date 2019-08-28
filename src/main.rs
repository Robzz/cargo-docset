use clap::{App, Arg, SubCommand, crate_authors, crate_version};
use cargo::{
    Config as CargoCfg,
    core::Workspace,
    util::important_paths::find_root_manifest_for_wd
};
use snafu::ResultExt;

use std::env::current_dir;

mod commands;
mod common;
mod error;

use common::Package;
use commands::generate::{generate, GenerateConfig};
use crate::error::*;

fn main() -> Result<()> {
    let matches =
        App::new("cargo-docset")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Generates a Zeal/Dash docset from a crate documentation.")
        .bin_name("cargo")
        .subcommand(
            SubCommand::with_name("docset")
            .arg(
                Arg::from_usage("-p, --package <SPEC>...  'Package(s) to document'")
                .required(false))
            .arg(
                Arg::from_usage("--exclude <SPEC>...  'Package(s) to exclude from the documentation'")
                .multiple(true)
                .required(false))
            .arg(
                Arg::from_usage("-v, --verbose  'Enable verbose output (-vv for extra verbosity).'")
                .multiple(true))
            .args_from_usage(
                "-q, --quiet                'Suppress all output to stdout.'
                --all                       'Document all packages in the workspace'
                --no-deps                   'Dont build documentation for dependencies'
                --document-private-items    'Document private items'
                "))
        .get_matches();
    let sub_matches = matches.subcommand_matches("docset").unwrap();

    let quiet = sub_matches.is_present("quiet");
    let verbosity_level = sub_matches.occurrences_of("verbose") as u32;

    if quiet && verbosity_level != 0 {
        eprintln!("Cannot specify `--quiet` with `--verbose`.");
        return Ok(())
    }

    let mut cargo_cfg = CargoCfg::default().context(Cargo)?;
    cargo_cfg.configure(verbosity_level, Some(quiet), &None, false, false, false, &None, &[]).context(Cargo)?;

    let mut cfg = GenerateConfig::default();
    cfg.no_dependencies = sub_matches.is_present("no-deps");
    cfg.package = if sub_matches.is_present("all") {
        Package::All
    }
    else if let Some(packages) = sub_matches.values_of_lossy("package") {
        Package::List(packages)
    }
    else if let Some(package) = sub_matches.value_of("package") {
        Package::Single(package.to_owned())
    }
    else { Package::Current };
    cfg.doc_private_items = sub_matches.is_present("document-private-items");
    cfg.exclude = sub_matches.values_of_lossy("exclude").unwrap_or_else(|| Vec::new());

    let cur_dir = current_dir().context(Io)?;
    let root_manifest = find_root_manifest_for_wd(&cur_dir).context(Cargo)?;
    let workspace = Workspace::new(&root_manifest, &cargo_cfg).context(Cargo)?;

    generate(&cargo_cfg, &workspace, cfg)?;

    Ok(())
}
