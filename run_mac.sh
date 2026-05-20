#!/bin/bash

# Quicko2 Mac Launcher
# This script builds and runs the relay server and the macOS UI.

set -e

echo "🚀 Building Quicko2 Core and Server..."
cargo build -p quicko2-server
cargo rustc -p quicko2-ffi --lib --crate-type cdylib -- -C link-arg=-undefined -C link-arg=dynamic_lookup

echo "📦 Preparing Python module..."
cp target/debug/libquicko2_ffi.dylib ui-macos/src/quicko2_core.so

echo "🌐 Starting Relay Server in background..."
./target/debug/quicko2-server &
SERVER_PID=$!

echo "🖥️ Starting macOS UI..."
export PYTHONPATH=ui-macos/src
python3 -m quicko_ui.app

# Cleanup on exit
kill $SERVER_PID
