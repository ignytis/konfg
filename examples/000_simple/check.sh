#!/usr/bin/env bash

set -e

echo "====== TEST START ====="
echo "Expected: "
cat examples/000_simple/out.yaml

echo ""
echo ""
echo "-----"
echo ""

echo "Actual: "
./target/debug/konfg \
    file-yaml://$PWD/examples/000_simple/in_0.yaml \
    file-yaml:///$PWD/examples/000_simple/in_1.yaml \
    \
    --param my.param=awesome

echo "====== TEST END ====="
