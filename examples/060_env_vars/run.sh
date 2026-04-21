#!/usr/bin/env bash

# This example demonstrates reading configuration from environment variables
# Usage:
#
# $ ./examples/060_env_vars/run.sh

set -eu

app_cmd="${APP_CMD:-cargo run --}"

# We set environment variables with a prefix MYAPP
# They will be merged into the configuration, overwriting values from config.yaml
export MYAPP__SERVER__PORT=9000
export MYAPP__DATABASE__USER=admin

${app_cmd} build \
  -i $PWD/examples/060_env_vars/config.yaml \
  -i env MYAPP \
  -o stdio json
