#!/bin/bash

# iOS Localization Tool Build Script
set -e

echo "Rosetta - Build Script"
echo

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "✗ Rust not found. Please install Rust:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "✓ Rust found"

# Check project directory
if [ ! -f "Cargo.toml" ]; then
    echo "✗ Please run this script from the project root directory"
    exit 1
fi

# Build project
echo "• Building project..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✓ Build successful"
    echo
    echo "Usage:"
    echo "  ./target/release/rosetta --help"
    echo "  ./target/release/rosetta ja --api-key YOUR_API_KEY"
    echo
    echo "Install to system (optional):"
    echo "  cargo install --path ."
    echo
else
    echo "✗ Build failed"
    exit 1
fi