#!/usr/bin/env bash

set -e

echo "====== TEST START ====="
echo "Expected Dotenv: "
cat examples/002_dotenv/out.env

echo ""
echo ""
echo "-----"
echo ""

echo "Actual Dotenv: "
./target/debug/konfg build \
    -i file examples/002_dotenv/in.env dotenv \
    --param dbname=mydb \
    -o dotenv

echo "====== TEST END ====="
