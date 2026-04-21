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
  - `stdio <format>`: Write to standard output as `<format>`.
  - `file <path> <format>`: Write to `<path>` as `<format>`.
  - `file <path>`: Write to `<path>`, detecting format by extension.
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

- `command(['arg1', 'arg2', ...])` - execute a system comand (_a pro tip: the output could be splitted using `lines` filter)
- `env(name, default = '')` - read an environment variable
- `md5(input)` - MD5 hash
- `sha256(input)` - SHA256 hash
- `sha512(input)` - SHA512 hash

### Filters

Standard Minijinja filters are available:
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

### More Examples

#### 1. Convert YAML to JSON
```bash
konfg build -i config.yaml -o stdio json
```

#### 2. Merge `.env` with Deep Nesting
`konfg` supports `__` as a separator for nested keys in `.env` files.

**`database.env`**
```env
DB__HOST=localhost
DB__PORT=5432
```

**Command**
```bash
konfg build -i database.env -o stdio json
```
**Output**
```json
{
  "db": {
    "host": "localhost",
    "port": "5432"
  }
}
```

#### 3. Use Jinja Functions
`konfg` provides several useful functions for your templates.

**`config.yaml`**
```yaml
app:
  version: "{{ env('APP_VERSION', '1.0.0') }}"
  secret_hash: "{{ sha256(my_secret) }}"
  files: {{ command(['ls', '-1', 'src']) | lines }}
```

**Command**
```bash
konfg build -i config.yaml -p my_secret=topsecret
```

---

## Supported Formats
- **YAML** (`.yaml`, `.yml`)
- **JSON** (`.json`)
- **TOML** (`.toml`)
- **Properties** (`.properties`)
- **dotenv** (`.env`)

## License
GPL-3.0
