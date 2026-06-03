#!/bin/bash
# Build script for boilerplate-cli-ui-rust

set -e

APP_NAME="boilerplate-cli-ui-rust"

echo "Building ${APP_NAME}..."

# Build with release optimizations
cargo build --release

echo "Built: target/release/${APP_NAME}"
ls -lh target/release/${APP_NAME}

echo ""
echo "Usage:"
echo "  ./target/release/${APP_NAME} start           # Start server with UI"
echo "  ./target/release/${APP_NAME} start -p 3000   # Start on custom port"
echo "  ./target/release/${APP_NAME} version         # Show version"
