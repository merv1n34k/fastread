# FastRead

## Overview
A speed reader that displays one word at a time with a red focal point letter for optimal recognition. Built with Rust + egui as a single lightweight binary.

## Commands
- `make setup` — install dependencies
- `make dev` — run in debug mode
- `make build` — release build (optimized, stripped)
- `make test` — run unit tests
- `make test-all` — run all tests
- `make lint` — clippy with warnings as errors
- `make fmt` — auto-format with rustfmt
- `make clean` — remove build artifacts

## Code Style
- Single-file app in `src/main.rs`
- No unnecessary comments or docstrings
- Theme colors stored in `Theme` struct, toggled at runtime
- Text preprocessing in `parse_words()` / `collapse_trailing_punct()`
