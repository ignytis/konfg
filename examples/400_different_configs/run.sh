#!/usr/bin/env bash
#
# Usage:
#
# $ ./examples/400_different_configs/run.sh

set -eu

app_cmd="${APP_CMD:-cargo run --}"

${app_cmd} build \
  -i file $PWD/examples/400_different_configs/00_vars.yaml yaml \
  -f stash push vars \
  -i file $PWD/examples/400_different_configs/10_regions.toml toml \
  -f stash push regions \
  -f stash pop vars _misc.vars \
  -f stash pop regions _misc.regions \
  -m env merge_by_key name \
  -i tplfile $PWD/examples/400_different_configs/05_defaults.yaml yaml \
  -i tplfile $PWD/examples/400_different_configs/90_deployment.yaml yaml \
  -f delete _misc \
  -o stdio yaml
