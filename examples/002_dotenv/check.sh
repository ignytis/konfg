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
./target/debug/konfg \
    file-dotenv://examples/002_dotenv/in.env \
    --param dbname=mydb \
    -o stdio-dotenv://

echo "====== TEST END ====="
