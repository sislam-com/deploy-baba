# Contributing to deploy-baba

Thank you for your interest in contributing to deploy-baba.

## Prerequisites

- Rust 1.75+ via [rustup](https://rustup.rs/)
- [just](https://github.com/casey/just) command runner
- Optional: `cargo-watch`, `cargo-audit`, `cargo-llvm-cov` for full workflow
- Optional: `cargo-lambda` for deployment only (`just deploy`)

```bash
cargo install cargo-watch cargo-audit cargo-llvm-cov
cargo install cargo-lambda --locked   # deploy-only; skip if not deploying to AWS
```

## Development Workflow

All developer commands go through the justfile:

```bash
just dev          # Format + lint + test (the inner loop)
just ui           # Run the portfolio site locally at http://localhost:3000
just quality      # Full quality gate before submitting a PR
just docs         # Build and open rustdoc
```

Run `just` with no arguments to see all available commands.

## Submitting Changes

1. Fork the repo and create a feature branch from `main`
2. Make your changes
3. Run `just quality` — this must pass clean
4. Open a pull request against `main`

## Code Standards

- Library crates use `thiserror` for errors, never `anyhow`
- All public items have `///` doc comments
- No `unwrap()` in library code
- Tests go in `#[cfg(test)] mod tests` at the bottom of each file
- Follow existing patterns — the codebase is consistent by design

## License

By contributing, you agree that your contributions will be licensed under the
MIT license.
