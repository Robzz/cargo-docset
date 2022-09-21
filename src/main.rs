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
    #[clap(long, value_parser)]
    /// Build documentation for the specified target triple.
    pub target: Option<String>,
    #[clap(long, value_parser)]
    /// Override the workspace target directory.
    pub target_dir: Option<PathBuf>,
    #[clap(long, action)]
    /// Do not clean the doc directory before generating the rustdoc.
    pub no_clean: bool,
    #[clap(long, action)]
    /// Document only this package's library.
    pub lib: bool,
    #[clap(long, value_parser)]
    /// Document only the specified binary.
    pub bin: Vec<String>,
    #[clap(long, action)]
    /// Document all binaries.
    pub bins: bool,
    #[clap(long, value_parser)]
    /// Specify or override the name of the docset, this is the display name used by your docset
    /// browser.
    pub docset_name: Option<String>,
    #[clap(long, value_parser, name("PACKAGE"))]
    /// Specify or override the package whose index will be used as the docset index page.
    pub docset_index: Option<String>,
    #[clap(long, value_parser)]
    /// Specify or override the docset platform family, this is used as the keyword you can specify
    /// in your docset browser search bar to search this specific docset).
    pub platform_family: Option<String>
}

impl DocsetParams {
    /// Generate args for the cargo doc invocation.
    fn into_args(self) -> Vec<String> {
        let mut args = Vec::new();
        if let Some(manifest_path) = self.manifest.manifest_path {
            args.push("--manifest-path".to_owned());
            args.push(manifest_path.to_string_lossy().to_string());
        }
        if self.workspace.workspace || self.workspace.all {
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
    //use std::{path::PathBuf, str::FromStr};

    use clap::Parser;

    use crate::{DocsetParams, Commands};

    use super::Cli;

    #[test]
    fn clap_verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }

    #[test]
    fn test_default_docset_params_into_args_is_empty() {
        let params = DocsetParams::default();
        let args = params.into_args();
        assert!(args.is_empty());
    }

    const TEST_DOCSET_PARAMS_1_MANIFEST_PATH: &str = "../somewhere_else/";
    const TEST_DOCSET_PARAMS_1_ENABLED_FEATURE: &str = "feature1";
    const TEST_DOCSET_PARAMS_1_EXCLUDED: &str = "excluded_package";
    const TEST_DOCSET_PARAMS_1_TARGET: &str = "x86_64-pc-windows-gnu";
    const TEST_DOCSET_PARAMS_1_DOCSET_NAME: &str = "Unit test docset 1";
    const TEST_DOCSET_PARAMS_1_DOCSET_INDEX: &str = "member1";
    const TEST_DOCSET_PARAMS_1_PLATFORM_FAMILY: &str = "dp1";
    const TEST_DOCSET_PARAMS_1_ARGS: &[&str] = &[
        "cargo",
        "docset",
        "--manifest-path",
        TEST_DOCSET_PARAMS_1_MANIFEST_PATH,
        "--workspace",
        "--no-default-features",
        "--features",
        TEST_DOCSET_PARAMS_1_ENABLED_FEATURE,
        "--exclude",
        TEST_DOCSET_PARAMS_1_EXCLUDED,
        "--no-deps",
        "--document-private-items",
        "--target",
        TEST_DOCSET_PARAMS_1_TARGET,
        "--no-clean",
        "--docset-name",
        TEST_DOCSET_PARAMS_1_DOCSET_NAME,
        "--docset-index",
        TEST_DOCSET_PARAMS_1_DOCSET_INDEX,
        "--platform-family",
        TEST_DOCSET_PARAMS_1_PLATFORM_FAMILY
    ];

    fn get_test_docset_params_1() -> Cli {
        Cli::parse_from(TEST_DOCSET_PARAMS_1_ARGS)
    }

    #[test]
    fn test_parse_docset_params_1() {
        let res = Cli::try_parse_from(TEST_DOCSET_PARAMS_1_ARGS);
        assert!(res.is_ok(), "Could not parse CLI arguments: {}", res.err().unwrap());
    }

    #[test]
    fn test_validate_matches_docset_params_1() {
        let params = get_test_docset_params_1();
        match params.command {
            Commands::Docset(params) => {
                assert!(params.manifest.manifest_path.is_some());
                assert_eq!(params.manifest.manifest_path.unwrap().to_string_lossy(), TEST_DOCSET_PARAMS_1_MANIFEST_PATH);

                assert!(params.workspace.workspace);
                assert!(params.workspace.package.is_empty());
                assert_eq!(params.workspace.exclude.len(), 1);
                assert_eq!(params.workspace.exclude[0], TEST_DOCSET_PARAMS_1_EXCLUDED);

                assert!(params.features.no_default_features);
                assert_eq!(params.features.features.len(), 1);
                assert_eq!(params.features.features[0], TEST_DOCSET_PARAMS_1_ENABLED_FEATURE);

                assert!(params.no_dependencies);

                assert!(params.doc_private_items);

                assert!(params.target.is_some());
                assert_eq!(params.target.unwrap(), TEST_DOCSET_PARAMS_1_TARGET);

                assert!(params.target_dir.is_none());

                assert!(params.no_clean);

                assert!(!params.lib);

                assert!(params.bin.is_empty());

                assert!(!params.bins);

                assert!(params.docset_name.is_some());
                assert_eq!(params.docset_name.unwrap(), TEST_DOCSET_PARAMS_1_DOCSET_NAME);

                assert!(params.docset_index.is_some());
                assert_eq!(params.docset_index.unwrap(), TEST_DOCSET_PARAMS_1_DOCSET_INDEX);

                assert!(params.platform_family.is_some());
                assert_eq!(params.platform_family.unwrap(), TEST_DOCSET_PARAMS_1_PLATFORM_FAMILY);
            }
        }
    }

    #[test]
    fn test_validate_docset_params_1_into_args() {
        let params = get_test_docset_params_1();
        match params.command {
            Commands::Docset(params) => {
                let mut cargo_doc_args = params.into_args();

                let expected_flags = &[
                    "--workspace",
                    "--no-default-features",
                    "--no-deps",
                    "--document-private-items",
                ];

                let expected_pairs = &[
                    ("--manifest-path", TEST_DOCSET_PARAMS_1_MANIFEST_PATH),
                    ("--features", TEST_DOCSET_PARAMS_1_ENABLED_FEATURE),
                    ("--exclude", TEST_DOCSET_PARAMS_1_EXCLUDED),
                    ("--target", TEST_DOCSET_PARAMS_1_TARGET)
                ];

                for flag in expected_flags {
                    assert!(cargo_doc_args.contains(&flag.to_string()), "Expected flag {} in vector {:?}", flag, cargo_doc_args);
                    let i = cargo_doc_args.iter().enumerate().find(|(_i, arg)| flag == arg).unwrap().0;
                    cargo_doc_args.remove(i);
                }

                for pair in expected_pairs {
                    let mut i = -1;
                    'inner: for (j, sub) in cargo_doc_args.chunks_exact(2).enumerate() {
                        if sub[0] == pair.0 && sub[1] == pair.1 {
                            i = j as i32;
                            break 'inner;
                        }
                    }

                    assert!(i >= 0, "Expected argument pair {:?} in arguments {:?}", pair, cargo_doc_args);
                }
            }
        }
    }
}
