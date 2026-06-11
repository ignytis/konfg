#!/usr/bin/env bash

# This example demonstrates the usage of custom merge strategies.
# Usage:
#
# $ ./examples/300_merge_strategies/run.sh

set -eu

app_cmd="${APP_CMD:-cargo run --}"

echo "=== 1. Default (simple) merge strategy ==="
echo "Without any merge strategies, maps are merged recursively and arrays are appended."
${app_cmd} build \
    -i file ./examples/300_merge_strategies/config_a.yaml \
    -i file ./examples/300_merge_strategies/config_b.yaml \
    -o stdio yaml

echo -e "\n=== 2. Using merge_by_key strategy ==="
echo "Specifying --merge-strategy / -m allows merging array elements matching a key instead of appending."
${app_cmd} build \
    -i file ./examples/300_merge_strategies/config_a.yaml \
    -i file ./examples/300_merge_strategies/config_b.yaml \
    -m app.features merge_by_key name \
    -o stdio yaml

echo -e "\n=== 3. Using overwrite strategy ==="
echo "Applying overwrite strategy forces the target property to be completely replaced by the source, rather than merged recursively. We can also rewrite or reset it afterwards."
${app_cmd} build \
    -i file ./examples/300_merge_strategies/config_a.yaml \
    -m app.database overwrite \
    -i file ./examples/300_merge_strategies/config_b.yaml \
    -m app.database simple \
    -o stdio yaml

echo -e "\n=== 4. Combining both strategies ==="
${app_cmd} build \
    -i file ./examples/300_merge_strategies/config_a.yaml \
    -i file ./examples/300_merge_strategies/config_b.yaml \
    -m app.features merge_by_key name \
    -m app.database overwrite \
    -o stdio yaml
