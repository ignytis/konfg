#!/usr/bin/env bash

# This example demonstrates using filters to manage temporary context (globals)
# and restructuring the final configuration.
# Usage:
#
# $ ./examples/070_filters/run.sh

set -eu

app_cmd="${APP_CMD:-cargo run --}"

${app_cmd} build \
  -i $PWD/examples/200_multipart_config/values.yaml \
  -f move . _values \
  -i file $PWD/examples/200_multipart_config/config_0_global.yaml \
  -i file $PWD/examples/200_multipart_config/config_1_env.yaml \
  -i file $PWD/examples/200_multipart_config/config_2_regional.yaml \
  -f delete _values \
  -o stdio yaml
