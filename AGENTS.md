# GEMINI.md - konfg

## Project Overview

`konfg` (konfig forged) is a Rust-based CLI tool designed to build and merge configuration files. It supports Jinja2 templates, allowing dynamic configuration values based on external parameters and values from previously merged files. It handles YAML, JSON, TOML, Properties, and Dotenv formats.

### Key Features

- **Multi-format Support:** Merge YAML, JSON, TOML, Properties (`.properties`), and Dotenv (`.env`) files.
- **Jinja2 Templating:** Power your configurations with `minijinja`. Access CLI parameters and previously merged configuration values within templates.
- **Deep Merging:** Intelligently merges nested objects. Arrays are replaced wholesale (later values win).
- **Flexible Input/Output:** Supports multiple input sources and output destinations via `stdio` and `file` handlers.
- **Filters:** Modify the merged configuration using filters (e.g., `delete` to remove specific parameters).
- **Format Auto-detection:** Automatically detects formats based on file extensions or explicitly provided tokens.

## Architecture

The project is structured into modular components:

- **`src/main.rs`:** Entry point, orchestrates the command execution.
- **`src/cli/`:** CLI parsing (via `clap`) and command implementations (e.g., `build`).
- **`src/handlers/format/`:** Implementations for parsing and serializing different configuration formats.
- **`src/workflow/io/`:** Handles reading from and writing to different targets (files, env, param, stdio).
- **`src/workflow/filters/`:** Implements filters to modify the merged configuration.
- **`src/workflow/`:** Workflow logic and the `Stage` structure.
- **`src/utils/`:** Utility functions for deep merging (`cfg_values`) and parameter handling (`hashmap`).
- **`src/jinja/`:** Jinja2 template rendering logic.

## Code style

- Format the code using `cargo fmt`
- Document the structures, functions, enums, constants and other types
- Avoid multiple assertions on a single instructions, like `if ... = let ... = fn()`
- Return fast

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
    file_format_handlers::{self, FormatHandler},
    workflow::io::{self, IoHandler},
    types::format::Format,
};
```

## Post-edit actions

At the end of plan execution:
1. Format the code using `cargo fmt`
2. Run unit tests using `cargo test`
3. Compile the application using `cargo build`

## Building and Running

### Prerequisites

- Rust (latest stable version, 2024 edition support).

### Key Commands

- **Build:** `cargo build`
- **Run:** `cargo run -- build -i <inputs> [options]`
- **Test:** `cargo test`

### Example Usage

```bash
cargo run -- build -i template1.yaml -i template2.toml -i param my.param value -o stdio json
```

## Development Conventions

- **Rust Edition:** Targets the 2024 edition of Rust.
- **Error Handling:** Uses `anyhow` for flexible and context-rich error reporting.
- **CLI Design:** Utilizes `clap` with the `derive` feature for a robust CLI interface.
- **Data Intermediary:** Uses `serde_json::Value` as a universal internal representation for all supported configuration formats during processing and merging.
- **Code Style:** Functionality is split into logical modules (`format_handlers`, `io`, `types`, `utils`). General-purpose logic should be kept separate from CLI-specific code.
- **Testing:** New features and bug fixes should include corresponding tests in their respective modules or as integration tests.
