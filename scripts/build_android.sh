#!/bin/bash
set -e

# Configuration
PROJECT_NAME="quicko2_ffi"
KOTLIN_OUT="ui-android/app/src/main/java"
PACKAGE_NAME="dev.quicko.core"

# Ensure tools are installed
if ! command -v cargo-ndk &> /dev/null; then
    echo "cargo-ndk not found. Please install with: cargo install cargo-ndk"
    exit 1
fi

# Build for host first to ensure we can generate bindings
echo "Building host library for binding generation..."
cargo build -p quicko2-ffi

# Determine dylib extension
OS=$(uname -s)
if [ "$OS" = "Darwin" ]; then
    DYLIB="target/debug/lib${PROJECT_NAME}.dylib"
else
    DYLIB="target/debug/lib${PROJECT_NAME}.so"
fi

# Generate Kotlin bindings
echo "Generating Kotlin bindings in $KOTLIN_OUT..."
cargo run -p quicko2-ffi --bin uniffi-bindgen -- generate --library "$DYLIB" --language kotlin --out-dir "$KOTLIN_OUT" --no-format

# Attempt Android build if NDK is present
if [ -n "$ANDROID_NDK_HOME" ] || [ -d "$HOME/Library/Android/sdk/ndk" ]; then
    echo "NDK detected, attempting Android build..."
    # You might want to add targets here
    # cargo ndk -t aarch64-linux-android build --release
    echo "Skipping full Android build in dry-run mode. Bindings are generated."
else
    echo "Warning: ANDROID_NDK_HOME not set. Skipping .so compilation for Android."
    echo "Kotlin bindings were generated using the host library."
fi

echo "Success! Bindings generated in $KOTLIN_OUT"
