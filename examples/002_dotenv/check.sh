#!/usr/bin/env bash

set -e

echo "Expected Dotenv: "
cat examples/002_dotenv/out.env

echo ""
echo ""
echo "-----"
echo ""

echo "Actual Dotenv: "
./target/debug/konfg \
    examples/002_dotenv/in.env \
    --param dbname=mydb \
    -o stdout-dotenv://
