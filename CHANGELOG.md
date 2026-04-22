## 0.1.0 - 2026-04-22

The first release.

- Input / output handlers
  - Files: Reads and writes files on local filesystem.
           See the "file format handlers" for more details about formats
  - Stdio: Uses standard input / output to parse contents in particular format.
           See the "file format handlers" for more details about formats
  - Env: Read-only. Reads the environment variables

- File format handlers
  - Java properties (`.properties`)
  - JSON (`.json`)
  - YAML (`.yaml`, `.yml`)
  - TOML (`.toml`)
  - Dotenv (`.env`)

- Jinja
  - Functions:
    - command - to execute system commands and process the output
    - env - to read environment variables
    - hashing (sha256, sha512, md5)