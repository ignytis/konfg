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
./target/debug/konfg \
    file-properties://$PWD/examples/001_properties/in.properties \
    \
    --param user=john \
    -o stdio-properties://

echo "====== TEST END ====="
