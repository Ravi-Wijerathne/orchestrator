#!/bin/bash

# File Orchestrator GUI Launcher

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "🗂️ File Orchestrator - GUI Launcher"
echo ""

# Check if GUI binary exists
if [ ! -f "$SCRIPT_DIR/target/release/fo" ]; then
    echo "❌ GUI binary not found!"
    echo "Please build with: cargo build --features gui --release"
    exit 1
fi

# Launch GUI
echo "🚀 Starting File Orchestrator GUI..."
"$SCRIPT_DIR/target/release/fo" --gui "$@"
