#!/usr/bin/env bash

# This example demonstrates usage of custom Jinja functions
# Usage:
#
# $ ./examples/040_jinja_tricks/run.sh

set -eu

app_cmd="${APP_CMD:-cargo run --}"

${app_cmd} build \
  -i $PWD/examples/040_jinja_tricks/config.yaml \
  -p my.api_key=sk_test_12345
