# GEMINI.md - konfg

## Project Overview

`konfg` (konfig forged) is a Rust-based CLI tool designed to build and merge configuration files. It supports Jinja2 templates, allowing dynamic configuration values based on external parameters and values from previously merged files. It handles YAML, JSON, TOML, Properties, and Dotenv formats.

### Key Features

- **Multi-format Support:** Merge YAML, JSON, TOML, Properties (`.properties`), and Dotenv (`.env`) files.
- **Jinja2 Templating:** Power your configurations with `minijinja`. Access CLI parameters and previously merged configuration values within templates.
- **Deep Merging:** Intelligently merges nested objects. Arrays are replaced wholesale (later values win).
- **Flexible Input/Output:** Supports URL-like schemes for specifying input and output formats and destinations (e.g., `stdin-yaml://`, `file-json://path/to/file.conf`, `stdout-toml://`).
- **Format Auto-detection:** Automatically detects formats based on file extensions or explicitly provided schemes.

## Architecture

The project is structured into modular components:

- **`src/main.rs`:** Entry point, orchestrates the processing loop (reading, rendering, and merging).
- **`src/cli.rs`:** CLI parsing (via `clap`) and parameter handling.
- **`src/format_handlers/`:** Implementations for parsing and serializing different configuration formats.
- **`src/io/`:** Handles reading from and writing to different targets (files, stdin/stdout).
- **`src/types/`:** Common types and format detection logic.
- **`src/utils/`:** Utility functions for deep merging, parameter handling, and URI parsing.
- **`src/jinja.rs`:** Jinja2 template rendering logic.
- **`src/output.rs`:** Logic for managing the final merged configuration output.

## Code style

- Format the code using `rustfmt`
- Document the structures, functions, enums, constants and other types

### Use statements

- Group the `use` statements using prefix and curly braces
- Group the `use` statements in the following order:
  - Standard library
  - Third-party
  - Local crate
- Separate groups with empty lines
- Order the `use` statements alphabetically
- Add `crate::` for local crates

Example:
```rust
use crate::{
    format_handlers::{self, FormatHandler},
    io::{self, IoHandler},
    types::format::Format,
    utils::uri::Uri
};
```

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
- **Code Style:** Functionality is split into logical modules (`format_handlers`, `io`, `types`, `utils`). General-purpose logic should be kept separate from CLI-specific code.
- **Testing:** New features and bug fixes should include corresponding tests in their respective modules or as integration tests.
