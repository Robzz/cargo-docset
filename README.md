# `cargo-docset` - Generate a Zeal/Dash docset for your Rust crate or workspace

![Build status](https://github.com/Robzz/cargo-docset/actions/workflows/rust.yml/badge.svg?branch=master)
[![Crate](https://img.shields.io/crates/v/cargo-docset.svg)](https://crates.io/crates/cargo-docset)

`cargo-docset` is a tool allowing you to generate a [Dash](https://kapeli.com/dash)/[Zeal](https://zealdocs.org/)
compatible docset for your Rust packages and their dependencies.

## Installation

`cargo-docset` depends on the SQLite3 library. You can either install the SQLite3 library on your system (see
[rusqlite's documentation](https://github.com/rusqlite/rusqlite#notes-on-building-rusqlite-and-libsqlite3-sys) for
help), or build the version that is bundled in the `libsqlite3-sys` crate by turning on the `bundled-sqlite` feature
flag when building `cargo-docset`.

You can install cargo docset with the usual cargo command: `cargo install cargo-docset`.

## How to use

Just run `cargo docset` in your crate's directory to generate the docset. It will be placed in the `target/docset`
directory. cargo-docset generally supports the same options as `cargo doc`, with a few additional ones. For more
information, run `cargo docset --help` or look below in this README.

To install your shiny new docset, copy it to your Zeal/Dash docset directory (available in the preferences, on Zeal at
least) and restart Zeal/Dash.

### Examples

Some more advanced examples:

* Include documentation only for some of the documented package's dependencies: `cargo docset --no-deps --package
  dependency1 --package dependency2 ...`
* Generate a docset for nightly Rust from the properly initialized (e.g. `git clone --recurse-submodules ...`) official
  Rust repository: `cargo +nightly docset --package std --package core --no-deps --docset-name "Rust nightly $(git rev-parse --short HEAD)" --docset-index std --platform-family rust-nightly`

### `cargo docset --help`

```
cargo-docset-docset
Generate a docset. This is currently the only available command, and should remain the default one
in the future if new ones are added

USAGE:
    cargo-docset docset [OPTIONS]

OPTIONS:
        --all-features
            Activate all available features

        --bin <BIN>
            Document only the specified binary

        --bins
            Document all binaries

        --docset-index <PACKAGE>
            Specify or override the package whose index will be used as the docset index page

        --docset-name <DOCSET_NAME>
            Specify or override the name of the docset, this is the display name used by your docset
            browser

        --document-private-items
            Generate documentation for private items

        --exclude <SPEC>
            Exclude packages from being processed

    -F, --features <FEATURES>
            Space-separated list of features to activate

    -h, --help
            Print help information

        --lib
            Document only this package's library

        --manifest-path <PATH>
            Path to Cargo.toml

        --no-clean
            Do not clean the doc directory before generating the rustdoc

        --no-default-features
            Do not activate the `default` feature

        --no-deps
            Do not document dependencies

    -p, --package <SPEC>
            Package to process (see `cargo help pkgid`)

        --platform-family <PLATFORM_FAMILY>
            Specify or override the docset platform family, this is used as the keyword you can
            specify in your docset browser search bar to search this specific docset)

        --target <TARGET>
            Build documentation for the specified target triple

        --target-dir <TARGET_DIR>
            Override the workspace target directory

        --workspace
            Process all packages in the workspace
```

## How it works

Currently, `cargo docset` runs `cargo` to generate the documentation, and then recursively walks the generated
directory. The contents of every file is inferred from the file path, and cargo-docset then fills a SQLite database with
the gathered information. The details of docset generation are available [here](https://kapeli.com/docsets#dashDocset).

`cargo-docset` does not (yet, at least) try to parse the generated documentation in any way, and therefore is limited in
the granularity of the index it can provide. In particular, the generated docset does not make use of the table of
contents feature.

Also, because `cargo-docset` walks through the whole `doc` directory, it must clear it before attempting to generate
the docset, in case there is some previously generated documentation that we don't want to pickup in the docset there.
You should probably not be storing anything of value in that directory anyway, but keep it in mind.

## Contributing

See [here](./CONTRIBUTING.md).
