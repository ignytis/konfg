# konfg

`konfg` (_Konfig ForGed_) is a powerful Rust-based CLI tool designed to build and merge configuration files. It supports **YAML**, **JSON**, **TOML** and **dotenv** formats and leverages **Jinja2** templating to allow dynamic configuration values.

## Key Features

- **Multi-format Support**: Seamlessly merge and convert between YAML, JSON, TOML and dotenv.
- **Deep Merging**: Intelligently merges nested objects. Later files overwrite values of earlier files.
- **Jinja2 Templating**: Use `minijinja` to power your configurations. Access CLI parameters and previously merged values directly within your templates.
- **Flexible I/O**: Support for URL-like schemes for specifying input/output formats and destinations (e.g., `stdin-yaml://`, `file-json://`, `stdout-toml://`, `file-dotenv://`).
- **Format Auto-detection**: Automatically detects formats based on file extensions or explicit URL schemes.

## Installation

Ensure you have Rust and Cargo installed, then build the project:

```bash
cargo build --release
```

## Usage

```bash
konfg <templates...> [options]
```

### Positional Arguments
- `<templates...>`: A list of paths or URLs to Jinja template files.

### Options
- `-p`, `--param <KEY=VALUE>`: Specify parameters available in Jinja context. Can be used multiple times. Supports dotted notation (e.g., `my.param=value`).
- `-o`, `--output <TARGET>`: Path or URL to the output file. (Default: `stdout-yaml://`).

### Input & Output URL Schemes

`konfg` uses a URI-like syntax to handle different formats and sources:

- `file:///path/to/file.json`: Detects format by extension.
- `file-yaml:///path/to/file.conf`: Forces YAML format regardless of extension.
- `stdin-toml://`: Reads standard input as TOML.
- `stdout-json://`: Writes the merged result to standard output as JSON.

## Merging Logic

1. **Deep Merge**: Nested maps are merged recursively.
2. **Overwriting**: If a key exists in multiple files, the value from the *later* file overwrites the earlier one.
3. **Template Context**:
   - Values passed via `--param` are available in all templates.
   - Scalar values (strings, numbers, booleans) from previously merged files are automatically added to the Jinja context for subsequent templates.

## Example

### `first.yaml`
```yaml
base_value: "hello"
some_dict:
    nested_key: "original"
```

### `second.yaml`
```yaml
derived_value: "{{ base_value }} world"
some_dict:
    nested_key: "overwritten"
    new_key: "{{ my.param }}"
```

### Command
```bash
konfg first.yaml second.yaml --param my.param=awesome
```

### Output (YAML)
```yaml
base_value: "hello"
derived_value: "hello world"
some_dict:
    nested_key: "overwritten"
    new_key: "awesome"
```

## Supported Formats
- **YAML** (`.yaml`, `.yml`)
- **JSON** (`.json`)
- **TOML** (`.toml`)
- *dotenv* (`.env`)

## License
GPL-3.0
