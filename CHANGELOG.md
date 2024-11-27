# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.2](https://github.com/rustic-rs/cdc/compare/v0.3.1...v0.3.2) - 2024-11-27

### Other

- update cross-ci workflow to use only stable Rust version
- *(deps)* add aws-lc-rs and aws-lc-sys to deny list due to build complexity and cross-compilation issues
- enable cross-compilation for additional targets in CI workflow to match rustic-rs
- enhance cross-checking job name with Rust version and update .gitignore for IDE files
- add installation script for default dependencies on x86_64-unknown-linux-musl
- add cross-ci ([#6](https://github.com/rustic-rs/cdc/pull/6))

## [0.3.1](https://github.com/rustic-rs/cdc/compare/v0.3.0...v0.3.1) - 2024-11-05

### Other

- use license in manifest

## [0.3.0](https://github.com/rustic-rs/cdc/compare/v0.2.1...v0.3.0) - 2024-11-05

### Fixed

- *(clippy)* [**breaking**] Fix clippy lints, update msrv and set edition to 2021.

## [0.2.1](https://github.com/rustic-rs/cdc/compare/v0.2.0...v0.2.1) - 2024-11-05

### Other

- update audit workflow
