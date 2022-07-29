# cargo-docset changelog

## Unreleased - v0.2.2

* Bugfix: module names are no longer suffixed by `::index`.
* Bugfix: fix several issues with virtual workspaces.
* Feature: add `--target-dir` option and respect `CARGO_TARGET_DIR` environment variable and `build.target_dir` config.
* Feature: add the `--docset-name` option in order to specify or override the docset name.
* Feature: add the `--docset-index` option in order to specify or override the docset index package.
* Refactored: use the [cargo-metadata](https://crates.io/crates/cargo_metadata) crate to obtain the workspace metadata,
  replace hand-rolled mechanisms.
* Maintenance: update dependencies to their latest versions.
* Maintenance: update to Rust edition 2021.

## 8/23/2020 - v0.2.1

* Bugfix: fix spelling of the `manifest-path` when passed down to cargo.
* Bugfix: fix detection of the workspace base directory for filesystem operations.
* Documentation: mention the dependency on SQLite and link to rusqlite's documentation in the README.
* Feature: provide the ability to use the SQLite version bundled with rusqlite through the `bundled-sqlite` feature.
* Maintenance: update `rusqlite` to v0.24.

## 6/22/2020 - v0.2.0

* Enhancement: do not depend on cargo anymore. This greatly improves the compile time, and should fix the recurring
  issues regarding the bundled version of cargo being unable to parse the Cargo.lock file. Drop other dependencies that
  are not needed anymore as a consequence.
* Feature: support the `--target` and `--manifest-path` options.
* Enhancement: enable default features by default, use the `--no-default-features` flag to disable this behavior.
* Maintenance: update the other dependencies to their latest versions.

## 6/19/2020 - v0.1.5

* Maintenance: update cargo to 0.42 (thanks to [@zgotch](https://github.com/zgotsch)) and run cargo update.

## 1/6/2020 - v0.1.4

* Bugfix: enable external JavaScript in Info.plist, should fix docsets not rendering properly in Dash.

## 10/28/2019 - v0.1.3

* Bugfix: don't crash the application when invoked directly as `cargo-docset`, print the usage message instead.
* Maintenance: run `cargo update`.

## 9/5/2019 - v0.1.2

* Feature: add the following command line options mimicking `cargo doc`: --features, --no-default-features,
  --all-features, --frozen, --locked, --offline, --lib, --bin and --bins.
* Feature: make cleaning the doc directory optional, through `--no-clean` option.
* Enhancement: use cargo clean command instead of `remove\_dir\_all` to clean the rustdoc directory.
* Enhancement: better error output.

## 8/29/2019 - v0.1.1

* Feature: add --exclude option
* Feature: add --quiet and --verbose options
* Enhancement: update dependencies to latest versions

## 8/14/2019 - v0.1.0

Initial release.
