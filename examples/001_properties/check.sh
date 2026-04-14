#!/usr/bin/env bash

set -e

echo "====== TEST START ====="
echo "Expected: "
cat examples/001_properties/out.properties

echo ""
echo ""
echo "-----"
echo ""

echo "Actual: "
./target/debug/konfg build \
    -i file $PWD/examples/001_properties/in.properties properties \
    \
    --param user=john \
    -o properties

echo "====== TEST END ====="
