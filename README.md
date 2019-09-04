# `cargo-docset` - Generate a Zeal/Dash docset for your Rust crate or workspace

[![Build Status](https://travis-ci.org/Robzz/cargo-docset.svg?branch=master)](https://travis-ci.org/Robzz/cargo-docset)

`cargo-docset` is a tool enabling you to generate a [Dash](https://kapeli.com/dash)/[Zeal](https://zealdocs.org/)
compatible docset for your Rust packages and their dependencies.

## How to use

Just run `cargo docset` in your crate's directory to generate the docset. It will be placed in the `target/docset`
directory. There are a few options to select which package(s) will be documented, check the help message for details.

To install your shiny new docset, copy it to your Zeal/Dash docset directory (available in the preferences, on Zeal at
least) and restart Zeal/Dash.

## How it works

Currently, `cargo-docset` runs `cargo` to generate the documentation, and then recursively walks the generated
directory. The contents of every file is inferred from the file path, and cargo-docset then fills a SQLite database with
the gathered information. The details of docset generation are available [here](https://kapeli.com/docsets#dashDocset).

`cargo-docset` does not (yet, at least) try to parse the generated documentation in any way, and therefore is limited in
the granularity of the index it can provide. In particular, the generated docset does not make use of the table of
contents feature.

Also, because `cargo-docset` walks through the whole `doc` directory, it must clear it before attempting to generate
the docset, in case there is some previously generated documentation that we don't want to pickup in the docset there.
You should probably not be storing anything of value in that directory anyway, but keep it in mind.

## Contributions

See [here](./CONTRIBUTING.md).
