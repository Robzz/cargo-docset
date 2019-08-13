# `cargo-docset` - Generate a Zeal/Dash docset for your Rust crate or workspace

`cargo-docset` is a tool enabling you to generate a [Dash](https://kapeli.com/dash)/[Zeal](https://zealdocs.org/)
compatible docset.

## How it works

Still WIP and not functional yet, but here's the plan:

Currently, `cargo-docset` runs `cargo` to generate the documentation, and then recursively walks the generated
directory. The contents of every file is inferred from the file path, and cargo-docset then fills a SQLite database with
the gathered information. The details of docset generation are available [here](https://kapeli.com/docsets#dashDocset).

`cargo-docset` does not (yet, at least) try to parse the generated documentation in any way, and therefore is limited in
the granularity of the index it can provide. In particular, the generated docset does not make use of the table of
contents feature.
