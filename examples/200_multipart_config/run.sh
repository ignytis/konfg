#!/usr/bin/env bash

# This example demonstrates demonstrates some techniques of re-using the configuration files
# Usage:
#
# $ ./examples/200_multipart_config/run.sh

set -eu

app_cmd="${APP_CMD:-cargo run --}"

${app_cmd} build \
  -i $PWD/examples/200_multipart_config/values.yaml \
  -f move . values \
  -f stash push --preserve values \
  -i file $PWD/examples/200_multipart_config/mixin.yaml \
  -f delete values \
  -f stash push mixin \
  -f stash pop values _values \
  -i file $PWD/examples/200_multipart_config/config_0_global.yaml \
  -i file $PWD/examples/200_multipart_config/config_1_env.yaml \
  -f stash pop mixin _imported_mixin \
  -i file $PWD/examples/200_multipart_config/config_2_regional.yaml \
  -f delete _values \
  -f delete _imported_mixin \
  -o stdio yaml
