#!/usr/bin/env bash

# This example demonstrates simple YAML to JSON conversion
# Usage:
#
# $ ./examples/010_basic_conversion/run.sh

set -eu

app_cmd="${APP_CMD:-cargo run --}"

${app_cmd} build -i $PWD/examples/010_basic_conversion/config.yaml -o stdio json
