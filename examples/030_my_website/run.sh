#!/usr/bin/env bash

# This example demonstrates re-use of previous files in context for next files.
# Base URL from `0_base.yaml` is propagated to API urls
# Usage:
# 
# Basic example - no additional parameters, use cargo run command
# $ ./examples/030_my_website/run.sh
#
# Use the compiled binary instead:
# $ APP_CMD=$PWD/target/debug/konfg ./examples/030_my_website/run.sh
#
# Add parameters to context:
# $ APP_CMD=$PWD/target/debug/konfg ./examples/030_my_website/run.sh --param debug_mode=1
#
# Set output to JSON instead of default YAML
# $ APP_CMD=$PWD/target/debug/konfg ./examples/030_my_website/run.sh --param debug_mode=1 -o stdio json

set -eu

args="\
    -i $PWD/examples/030_my_website/0_base.properties \
    -i $PWD/examples/030_my_website/1_backend.yaml \
    -i $PWD/examples/030_my_website/2_frontend.yaml \
"

app_cmd="${APP_CMD:-cargo run --}"

${app_cmd} build ${args} $@