#!/usr/bin/env bash

# This example demonstrates demonstrates some techniques of re-using the configuration files
# Usage:
#
# $ ./examples/200_multipart_config/run.sh

set -eu

app_cmd="${APP_CMD:-cargo run --}"

# This variable will be injected into config
export MY_EXAMPLE__FEATURE_FLAGS__MY_BLEEDING_EDGE_FEATURE="${MY_EXAMPLE__FEATURE_FLAGS__MY_BLEEDING_EDGE_FEATURE:-1}"

# The arguments are decomposed into multiple stages here - just to make commenting between stages possible
# 1. Load values file (plain config. No Jinja templating)
args="-i file ./examples/200_multipart_config/values.yaml"
# 1a. Add some environment variables into values
args="$args -i env MY_EXAMPLE"
# 2. Save the processed config to stash with name 'values'. Do NOT clean the current config.
args="$args -f stash push --preserve values"
# 3. Move the contants of values file from root level into 'values' attribute
args="$args -f move . values"
# 4. Read a mixin template file which uses values
args="$args -i tplfile ./examples/200_multipart_config/mixin.yaml"
# 5. Delete the previously moved 'values' section from config, keep mixin contents only
args="$args -f delete values"
# 6. Move the mixin to stash with name 'mixin'. No --preserve flag this time,
#    so configuration is not a blank value
args="$args -f stash push mixin"
# 7. Extract values from stash and save them to _values attribute of config.
#    Also no --preserve flag here, so there is no "values" in stash anymore
args="$args -f stash pop values _values"
# 8. Render configuration files and merge them into current config.
args="$args -i tplfile ./examples/200_multipart_config/config_0_global.yaml"
args="$args -i tplfile ./examples/200_multipart_config/config_1_env.yaml"
# 9. Previous configs didn't use mixin, but the next config will need it. Pop mixin back
args="$args -f stash pop mixin _imported_mixin"
# 10. Render a template which uses mixin
args="$args -i tplfile ./examples/200_multipart_config/config_2_regional.yaml"
# 11. Cleanup. Delete all the helper attributes
args="$args -f delete _values"
args="$args -f delete _imported_mixin"
# 12. Print the config
args="$args -o stdio yaml"

${app_cmd} build $args
