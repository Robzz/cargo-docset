# cargo-docset changelog

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
