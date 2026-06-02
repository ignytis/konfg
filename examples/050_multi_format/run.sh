#!/usr/bin/env bash

# This example demonstrates merging multiple configuration formats (TOML, YAML, Properties)
# Usage:
#
# $ ./examples/050_multi_format/run.sh

set -eu

app_cmd="${APP_CMD:-cargo run --}"

${app_cmd} build \
  -i tplfile $PWD/examples/050_multi_format/0_db.toml \
  -i tplfile $PWD/examples/050_multi_format/1_server.yaml \
  -i tplfile $PWD/examples/050_multi_format/2_app.properties \
  -f delete filter_test.delete_me \
  -o stdio yaml
