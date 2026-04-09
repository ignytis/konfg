# GEMINI.md - konfg

## Project Overview

`konfg` (konfig forged) is a Rust-based CLI tool designed to build and merge configuration files. It supports Jinja2 templates, allowing dynamic configuration values based on external parameters and values from previously merged files. It handles YAML, JSON, and TOML formats.

### Key Features

- **Multi-format Support:** Seamlessly merge YAML, JSON, and TOML files.
- **Jinja2 Templating:** Power your configurations with `minijinja`. Access CLI parameters and previously merged configuration values within templates.
- **Deep Merging:** Intelligently merges nested objects. Arrays are replaced wholesale (later values win).
- **Flexible Input/Output:** Supports URL-like schemes for specifying input and output formats and destinations (e.g., `stdin-yaml://`, `file-json://path/to/file.conf`, `stdout-toml://`).
- **Format Auto-detection:** Automatically detects formats based on file extensions or explicitly provided schemes.

## Architecture

The project is structured into modular components:

- **`src/main.rs`:** Entry point, CLI parsing (via `clap`), parameter handling, and the core processing loop (reading, rendering, and merging).
- **`src/input.rs`:** Defines supported formats and handles the parsing of raw content into `serde_json::Value`.
- **`src/merge.rs`:** Implements the deep merge logic for `serde_json::Value` objects.
- **`src/output.rs`:** Handles formatting the final merged configuration and writing it to the specified target.

## Code style

- Format the code using `rustfmt`
- Document the strctures, functions, enums, constants and other types

## Building and Running

### Prerequisites

- Rust (latest stable version, 2024 edition support).

### Key Commands

- **Build:** `cargo build`
- **Run:** `cargo run -- <templates> [options]`
- **Test:** `cargo test`

### Example Usage

```bash
cargo run -- template1.yaml template2.toml -p my.param=value -o stdout-json://
```

## Development Conventions

- **Rust Edition:** Targets the 2024 edition of Rust.
- **Error Handling:** Uses `anyhow` for flexible and context-rich error reporting.
- **CLI Design:** Utilizes `clap` with the `derive` feature for a robust CLI interface.
- **Data Intermediary:** Uses `serde_json::Value` as a universal internal representation for all supported configuration formats during processing and merging.
- **Code Style:** Functionality is split into logical modules (`input`, `merge`, `output`). General-purpose logic should be kept separate from CLI-specific code.
- **Testing:** New features and bug fixes should include corresponding tests in their respective modules or as integration tests.
