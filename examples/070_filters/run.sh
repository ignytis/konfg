#!/usr/bin/env bash

# This example demonstrates using filters to manage temporary context (globals)
# and restructuring the final configuration.
# Usage:
#
# $ ./examples/070_filters/run.sh

set -eu

app_cmd="${APP_CMD:-cargo run --}"

${app_cmd} build \
  -i tplfile $PWD/examples/070_filters/globals.yaml \
  -f move . _globals \
  -i tplfile $PWD/examples/070_filters/config.yaml \
  -f delete user.password \
  -f delete _globals \
  -o stdio json
