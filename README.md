# konfg

`konfg` (_Konfig ForGed_) is a powerful Rust-based CLI tool designed to build and merge configuration files. It supports **YAML**, **JSON**, **TOML**, **properties**, and **dotenv** formats and leverages **Jinja2** templating to allow dynamic configuration values.

## Key Features

- **Multi-format Support**: Seamlessly merge and convert between YAML, JSON, TOML, properties, and dotenv.
- **Deep Merging**: Intelligently merges nested objects. Later files overwrite values of earlier files.
- **Jinja2 Templating**: Use `minijinja` to power your configurations. Access CLI parameters and previously merged values directly within your templates.
- **Filters**: Modify the merged configuration using filters, such as deleting specific parameters.
- **Flexible I/O**: Support for multiple input sources and output destinations via `stdio` and `file` handlers.
- **Format Auto-detection**: Automatically detects formats based on file extensions or explicit CLI tokens.

## Try me in Docker!

There is a [Docker image](https://hub.docker.com/r/ignytis/konfg) built for each Git tag. You can use it:
- To quickly try the application in order to understand if you need it at all
- To run it in Init container inside Kubernetes cluster to build configuration for your app

_The commands below have networking disabled, so you can be sure that no side magic happens on app execution_

The first example, zero-file (standard input-output only):

```bash
echo -n '{"category": {"some_key": "some value"}}' | \
  docker run --rm \
    --network none \
    -i ignytis/konfg:latest \
    build \
      -i stdio json \
      -o stdio toml

# Output
[category]
some_key = "some value"
```

Let's enrich it with configuration from environment:

```bash
echo -n '{"category": {"some_key": "some value"}}' | \
  docker run --rm \
    --network none \
    -e EXAMPLE__CATEGORY__ENV_VAR=env_value \
    -i ignytis/konfg:latest\
      build \
        -i stdio json \
        -i env EXAMPLE \
        -o stdio toml

# Output
[category]
env_var = "env_value"
some_key = "some value"
```

And finally, a zero-file example with parameters and filters:

```bash
docker run --rm \
  --network none \
  ignytis/konfg:latest \
    build \
      -i param server.host 0.0.0.0 \
      -i param server.port 8080 \
      -i param temp.debug true \
      -f move server config \
      -f delete temp \
      -o stdio json

# Output
{
  "config": {
    "host": "0.0.0.0",
    "port": "8080"
  }
}
```

The next example uses config from this repository, so please clone it before running the command.

```bash
docker run --rm \
  --network none \
  -v $PWD/examples:/examples \
  -it ignytis/konfg:latest \
    build \
      -i /examples/010_basic_conversion/config.yaml \
      -o stdio json

# Output
{
  "database": {
    "url": "postgresql://localhost:5432/db"
  },
  "server": {
    "host": "0.0.0.0",
    "port": 8080
  }
}
```

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

- `-i`, `--input <args...>`: Input specification. Can be used multiple times.
  - `cmd <format> <cmd...>`: Execute a system command. Parse the output as `<format>`.
  - `env <PREFIX>`: read environment variables. Double underscore `__` is separator.
    Example: `MY_APP__TOP_LEVEL__SUB_LEVEL__MYVAR` will be processed is prefix is `MY_APP`
  - `file <path> <format>`: Read from `<path>`, render as Jinja template and parse as `<format>`.
  - `file <path>`: Read from `<path>`, detecting format by extension.
  - `param <key> <value>`: inject a single parameter. Use dots `.` for nested levels.
     Use double dots `..` to escape the dots.
  - `stdio <format>`: Read from standard input as `<format>`.
  - `tplfile <path> <format>`: Read from `<path>`, render as Jinja template and parse as `<format>`.
  - `tplfile <path>`: Read from `<path>`, detecting format by extension.
  - `<path>`: Shorthand for `tplfile <path>`.
- `-o`, `--output <args...>`: Output specification. (Default: `stdio yaml`).
  - `stdio <format>`: Write to standard output as `<format>`.
  - `file <path> <format>`: Write to `<path>` as `<format>`.
  - `file <path>`: Write to `<path>`, detecting format by extension.
  - `noop`: No Operation. Doesn't write anything; for testing purposes only.
- `-f`, `--filter <args...>`: Filter specification. Can be used multiple times.
  - `delete <key>`: Remove a parameter from the configuration. Use dots `.` for nested levels.
  - `move <source> <destination>`: Move a parameter from `<source>` to `<destination>`. Use `.` for the root level.
- `-m`, `--merge-strategy <args...>`: Merge strategy specification. Can be used multiple times.
  - `<path> <strategy> [strategy_args...]`
    Apply a specific merging strategy to the given path in the configuration structure.
    The strategy takes effect for all subsequent `-i` inputs until changed or reset.
    Place `-m` **before** the `-i` input it should affect.
    - `<path>`: The path in the merged configuration where the strategy is applied (e.g., `app.features`).
    - `<strategy>`:
      - `simple` (default): Recursive merge for maps, append for arrays.
      - `overwrite`: Overwrite target value completely instead of recursively merging.
      - `merge_by_key <key_name>`: Merge array elements by matching their `<key_name>` property.

## Merging Logic

1. **Deep Merge**: Nested maps are merged recursively (the `simple` strategy).
2. **Overwriting**: If a key exists in multiple files, the value from the *later* file overwrites the earlier one.
3. **Merge Strategies**: You can configure custom merge behaviors at specific paths using the `-m` / `--merge-strategy` option:
   - **`overwrite`**: Replaces the target collection with the source collection entirely.
   - **`merge_by_key <key>`**: Merges elements of arrays of objects by checking if their `<key>` value is equal, and recursively merging them.
4. **Template Context**:
   - Results of processing the previous inputs are available in context of next inputs
   - Scalar values (strings, numbers, booleans) from previously merged files are automatically added to the Jinja context for subsequent templates.

## Jinja

### Functions

Minijinja builtins are enabled; see the document:
https://docs.rs/minijinja/latest/minijinja/functions/index.html#built-in-functions

In addition, the following functions are defined:

- `command(['arg1', 'arg2', ...])` - execute a system command (_a pro tip: the output could be split using `lines` filter)
- `env(name, default = '')` - read an environment variable
- `md5(input)` - MD5 hash
- `sha256(input)` - SHA256 hash
- `sha512(input)` - SHA512 hash

### Filters

Standard Minijinja filters are available:
https://docs.rs/minijinja/latest/minijinja/filters/index.html#functions


### Tests

See Minijinja for standard tests like `is defined`:
https://docs.rs/minijinja/latest/minijinja/tests/index.html


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
env_value: {{ env('MY_ENV_VAR', 'default')  }}
```

### Command
```bash
MY_ENV_VAR=this_is_env \
  konfg build \
    -i first.yaml \
    -i param my.param awesome \
    -i second.yaml
```

### Output (YAML)
```yaml
base_value: hello
derived_value: hello world
env_value: this_is_env
my:
  param: awesome
some_dict:
  nested_key: overwritten
  new_key: awesome
```

### More Examples

#### 1. Convert YAML to JSON

**`config.yaml`**
```yaml
app:
  name: "myapp"
  port: 8080
```

**Command**
```bash
konfg build -i config.yaml -o stdio json
```

**Output**
```json
{
  "app": {
    "name": "myapp",
    "port": 8080
  }
}
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
  secret_hash: "{{ sha256('topsecret') }}"
  files: {{ command(['ls', '-1', 'src']) | lines }}
```

**Command**
```bash
konfg build -i config.yaml
```

**Output**
```yaml
app:
  files:
  - cli
  - handlers
  - jinja
  - main.rs
  - types
  - utils
  secret_hash: 53336a676c64c1396553b2b7c92f38126768827c93b64d9142069c10eda7a721
  version: 1.0.0
```

#### 4. Merge from Environment Variables
You can use the `env` input handler to read all environment variables with a specific prefix. It supports nested structures using `__` as a separator.

**Command**
```bash
export MYAPP__SERVER__PORT=9000
export MYAPP__DB__USER=admin
konfg build -i config.yaml -i env MYAPP -o stdio json
```

**Output**
```json
{
  "app": {
    "files": [
      "cli",
      "handlers",
      "jinja",
      "main.rs",
      "types",
      "utils"
    ],
    "secret_hash": "53336a676c64c1396553b2b7c92f38126768827c93b64d9142069c10eda7a721",
    "version": "1.0.0"
  },
  "db": {
    "user": "admin"
  },
  "server": {
    "port": "9000"
  }
}
```

#### 5. Use Filters
You can remove sensitive or unnecessary data from the final configuration using filters.

**Command**
```bash
konfg build \
  -i config.yaml \
  -f delete server.host \
  -o stdio json
```

**Output**
```json
{
  "database": {
    "url": "postgresql://localhost:5432/db"
  },
  "server": {
    "port": 8080
  }
}
```

#### 6. Use Merge Strategies
You can control how fields are merged using custom merge strategies.

**`config_a.yaml`**
```yaml
app:
  features:
    - name: "feature1"
      enabled: false
  database:
    host: "localhost"
    port: 5432
```

**`config_b.yaml`**
```yaml
app:
  features:
    - name: "feature1"
      enabled: true
  database:
    port: 5433
```

**Command**
```bash
konfg build \
  -i config_a.yaml \
  -m app.features merge_by_key name \
  -m app.database overwrite \
  -i config_b.yaml \
  -o stdio json
```

**Output**
```json
{
  "app": {
    "features": [
      {
        "name": "feature1",
        "enabled": true
      }
    ],
    "database": {
      "port": 5433
    }
  }
}
```

---

## Supported inputs and outputs:

- Files (see `Supported formats` below)
- Stdin/Stdout
- Environment variables (input only)

## Supported Formats

- **YAML** (`.yaml`, `.yml`)
- **JSON** (`.json`)
- **TOML** (`.toml`)
- **Properties** (`.properties`)
- **Dotenv** (`.env`)

## License
GPL-3.0
