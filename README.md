# konfg

`konfg` (_Konfig ForGed_) is a powerful Rust-based CLI tool designed to build and merge configuration files. It supports **YAML**, **JSON**, **TOML**, **properties**, and **dotenv** formats and leverages **Jinja2** templating to allow dynamic configuration values.

## Key Features

- **Multi-format Support**: Seamlessly merge and convert between YAML, JSON, TOML, properties, and dotenv.
- **Deep Merging**: Intelligently merges nested objects. Later files overwrite values of earlier files.
- **Jinja2 Templating**: Use `minijinja` to power your configurations. Access CLI parameters and previously merged values directly within your templates.
- **Flexible I/O**: Support for multiple input sources and output destinations via `stdio` and `file` handlers.
- **Format Auto-detection**: Automatically detects formats based on file extensions or explicit CLI tokens.

## Installation

Ensure you have Rust and Cargo installed, then build the project:

```bash
cargo build --release
```

## Usage

```bash
konfg build [options]
```

### Options

- `-i`, `--in <tokens...>`: Input specification. Can be used multiple times.
  - `stdio <format>`: Read from standard input as `<format>`.
  - `file <path> <format>`: Read from `<path>` as `<format>`.
  - `file <path>`: Read from `<path>`, detecting format by extension.
  - `<path>`: Shorthand for `file <path>`.
- `-o`, `--out <tokens...>`: Output specification. (Default: `stdio yaml`).
  - `<format>`: Shorthand for `stdio <format>`.
  - `stdio <format>`: Write to standard output as `<format>`.
  - `file <path> <format>`: Write to `<path>` as `<format>`.
- `-p`, `--param <KEY=VALUE>`: Specify parameters available in Jinja context. Can be used multiple times. Supports dotted notation (e.g., `my.param=value`).

## Merging Logic

1. **Deep Merge**: Nested maps are merged recursively.
2. **Overwriting**: If a key exists in multiple files, the value from the *later* file overwrites the earlier one.
3. **Template Context**:
   - Values passed via `--param` are available in all templates.
   - Scalar values (strings, numbers, booleans) from previously merged files are automatically added to the Jinja context for subsequent templates.

## Jinja

### Funcions

Minijinja builtins are enabled; see the document:
https://docs.rs/minijinja/latest/minijinja/functions/index.html#built-in-functions

In addition, the following functions are defined:

- `env(name, default = '')` - read an environment variable

### Filters

Standarad Minijinja filters are available:
https://docs.rs/minijinja/latest/minijinja/filters/index.html#functions


### Tests

See Minijinja for standard tests like `is defined`:
https://docs.rs/minijinja/latest/minijinja/filters/index.html#functions


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
konfg build -i first.yaml -i second.yaml -p my.param=awesome
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
- **Properties** (`.properties`)
- **dotenv** (`.env`)

## License
GPL-3.0
