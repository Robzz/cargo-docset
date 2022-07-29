use std::path::PathBuf;

use clap::{Parser, Subcommand, Args};

mod commands;
mod error;
mod io;

use crate::error::*;
use commands::generate::generate_docset;

#[derive(Debug, Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Commands
}

#[derive(Args, Default, Debug, Clone)]
/// Generate a docset. This is currently the only available command, and should remain the
/// default one in the future if new ones are added.
pub struct DocsetParams {
    #[clap(flatten)]
    pub manifest: clap_cargo::Manifest,
    #[clap(flatten)]
    pub workspace: clap_cargo::Workspace,
    #[clap(flatten)]
    features: clap_cargo::Features,
    #[clap(long("no-deps"))]
    /// Do not document dependencies.
    pub no_dependencies: bool,
    #[clap(long("document-private-items"))]
    /// Generate documentation for private items.
    pub doc_private_items: bool,
    #[clap(value_parser)]
    /// Build documentation for the specified target triple.
    pub target: Option<String>,
    #[clap(value_parser)]
    /// Override the workspace target directory.
    pub target_dir: Option<PathBuf>,
    #[clap(long, action)]
    /// Do not clean the doc directory before generating the rustdoc.
    pub no_clean: bool,
    #[clap(long, action)]
    /// Document only this package's library.
    pub lib: bool,
    #[clap(value_parser)]
    /// Document only the specified binary.
    pub bin: Vec<String>,
    #[clap(long, action)]
    /// Document all binaries.
    pub bins: bool,
    #[clap(long, value_parser)]
    /// Specify or override the docset name.
    pub docset_name: Option<String>,
    #[clap(long, value_parser)]
    /// Specify or override the package whose index will be used as the docset index page.
    pub docset_index: Option<String>,
    #[clap(long, value_parser)]
    /// Specify or override the docset platform family (used to specify the keyword used to search
    /// this specific docset in documentation browsers).
    pub platform_family: Option<String>
}

impl DocsetParams {
    /// Generate args for the cargo doc invocation.
    fn into_args(self) -> Vec<String> {
        let mut args = Vec::new();
        if self.workspace.all {
            args.push("--workspace".to_owned());
            for exclude in self.workspace.exclude {
                args.extend_from_slice(&["--exclude".to_owned(), exclude]);
            }
        } else {
            for package in self.workspace.package {
                args.extend_from_slice(&["--package".to_owned(), package]);
            }
        }
        if self.no_dependencies {
            args.push("--no-deps".to_owned())
        }
        if self.doc_private_items {
            args.push("--document-private-items".to_owned())
        }
        if !self.features.features.is_empty() {
            args.push("--features".to_owned());
            args.extend(self.features.features);
        }
        if self.features.no_default_features {
            args.push("--no-default-features".to_owned())
        }
        if self.features.all_features {
            args.push("--all-features".to_owned())
        }
        if let Some(target) = self.target {
            args.push("--target".to_owned());
            args.push(target);
        }
        if let Some(target_dir) = self.target_dir {
            args.push("--target-dir".to_owned());
            args.push(target_dir.to_string_lossy().to_string());
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

#[derive(Debug, Subcommand)]
enum Commands {
    Docset(DocsetParams)
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Docset(params) => {
            generate_docset(params)
        }
    }
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        io::error(&e.to_string())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_config_into_args() {
    }
}
