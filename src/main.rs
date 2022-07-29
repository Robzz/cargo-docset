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
    /// Generate a docset. This is currently the only available command, and should remain the
    /// default one in the future if new ones are added.
    Docset {
        #[clap(flatten)]
        manifest: clap_cargo::Manifest,
        #[clap(flatten)]
        workspace: clap_cargo::Workspace,
        #[clap(flatten)]
        features: clap_cargo::Features,
        #[clap(long)]
        /// Do not document dependencies.
        no_deps: bool,
        #[clap(long("document-private-items"))]
        /// Generate documentation for private items.
        doc_private_items: bool,
        #[clap(value_parser)]
        /// Build documentation for the specified target triple.
        target: Option<String>,
        #[clap(value_parser)]
        /// Override the workspace target directory.
        target_dir: Option<PathBuf>,
        #[clap(long, action)]
        /// Do not clean the doc directory before generating the rustdoc.
        no_clean: bool,
        #[clap(long, action)]
        /// Document only this package's library.
        lib: bool,
        #[clap(value_parser)]
        /// Document only the specified binary.
        bin: Vec<String>,
        #[clap(long, action)]
        /// Document all binaries.
        bins: bool,
        #[clap(long, value_parser)]
        /// Specify or override the docset name.
        docset_name: Option<String>,
        #[clap(long, value_parser)]
        /// Specify or override the package whose index will be used as the docset index page.
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
            no_clean,
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
                no_clean,
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
