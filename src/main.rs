use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod commands;
mod common;
mod error;

use crate::error::*;
use commands::generate::{generate, GenerateConfig};

#[derive(Debug, Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Commands
}

#[derive(Debug, Subcommand)]
enum Commands {
    Docset {
        #[clap(flatten)]
        manifest: clap_cargo::Manifest,
        #[clap(flatten)]
        workspace: clap_cargo::Workspace,
        #[clap(long)]
        no_deps: bool,
        #[clap(long("document-private-items"))]
        doc_private_items: bool,
        #[clap(flatten)]
        features: clap_cargo::Features,
        #[clap(value_parser)]
        target: Option<String>,
        #[clap(value_parser)]
        target_dir: Option<PathBuf>,
        #[clap(long, action)]
        clean: bool,
        #[clap(long, action)]
        lib: bool,
        #[clap(value_parser)]
        bin: Vec<String>,
        #[clap(long, action)]
        bins: bool,
        #[clap(long, value_parser)]
        docset_name: Option<String>,
        #[clap(long, value_parser)]
        docset_index: Option<String>
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Docset {
            manifest,
            workspace,
            no_deps: no_dependencies,
            doc_private_items,
            features,
            target,
            target_dir,
            clean,
            lib,
            bin,
            bins,
            docset_name,
            docset_index
        } => {
            generate(GenerateConfig {
                manifest,
                workspace,
                no_dependencies,
                doc_private_items,
                features: features.features,
                no_default_features: features.no_default_features,
                all_features: features.all_features,
                target,
                target_dir,
                clean,
                lib,
                bin,
                bins,
                docset_name,
                docset_index
            })
        }
    }
}

fn main() {
    let cli = Cli::parse();
    run(cli).unwrap();
}
