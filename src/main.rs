use clap::{App, Arg, SubCommand};
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
        .version("0.1")
        .author("Robin Chavignat")
        .about("Generates a Zeal/Dash docset from a crate documentation.")
        .bin_name("cargo")
        .subcommand(
            SubCommand::with_name("docset")
            .arg(
                Arg::from_usage(" -p, --package <SPEC>... 'Package(s) to document'")
                .required(false))
            .args_from_usage(
                "--all                      'Document all packages in the workspace'
                --no-deps                   'Dont build documentation for dependencies'
                --document-private-items    'Document private items'
                "))
        .get_matches();
    let sub_matches = matches.subcommand_matches("docset").unwrap();

    let mut cargo_cfg = CargoCfg::default().context(Cargo)?;
    cargo_cfg.configure(0, Some(false), &None, false, false, false, &None, &[]).context(Cargo)?;

    let mut cfg = GenerateConfig::default();
    cfg.no_dependencies = sub_matches.is_present("no-deps");
    cfg.package = if sub_matches.is_present("all") {
        Package::All
    }
    else if let Some(package) = sub_matches.value_of("package") {
        Package::Single(package.to_owned())
    }
    else { Package::Current };

    let cur_dir = current_dir().context(Io)?;
    let root_manifest = find_root_manifest_for_wd(&cur_dir).context(Cargo)?;
    let workspace = Workspace::new(&root_manifest, &cargo_cfg).context(Cargo)?;

    generate(&cargo_cfg, &workspace, cfg)?;

    Ok(())
}
