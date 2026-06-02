#!/usr/bin/env bash

set -euo pipefail

find examples -name 'run.sh' | while read script
do
    echo "Running $script..."
    bash $script
done