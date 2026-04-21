#!/usr/bin/env bash

# This example demonstrates merging multiple .env files with nested structure
# Usage:
#
# $ ./examples/020_dotenv_merge/run.sh

set -eu

app_cmd="${APP_CMD:-cargo run --}"

${app_cmd} build \
  -i $PWD/examples/020_dotenv_merge/app.env \
  -i $PWD/examples/020_dotenv_merge/overrides.env \
  -o stdio json
