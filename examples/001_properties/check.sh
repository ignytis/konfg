#!/usr/bin/env bash

set -e

echo "Expected: "
cat examples/001_properties/out.properties

echo ""
echo ""
echo "-----"
echo ""

echo "Actual: "
./target/debug/konfg \
    file://$PWD/examples/001_properties/in.properties \
    \
    --param user=john \
    -o stdout-properties://
