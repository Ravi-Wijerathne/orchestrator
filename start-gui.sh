#!/bin/bash

# File Orchestrator GUI Launcher
# Automatically starts the GUI version with all dependencies checked

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo -e "${BLUE}==================================================${NC}"
echo -e "${BLUE}  File Orchestrator - GUI Launcher${NC}"
echo -e "${BLUE}==================================================${NC}"
echo ""

# Function to print status messages
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Navigate to project directory
cd "$SCRIPT_DIR"

# Check if GUI binary exists
GUI_BINARY="$SCRIPT_DIR/target/release/fo"

if [ -f "$GUI_BINARY" ]; then
    # Check if it was built with GUI features
    print_status "Checking if GUI is available..."
    if ldd "$GUI_BINARY" 2>/dev/null | grep -q "webkit2gtk\|gtk" || otool -L "$GUI_BINARY" 2>/dev/null | grep -q "WebKit"; then
        print_success "GUI version found!"
        print_status "Starting File Orchestrator GUI..."
        echo ""
        exec "$GUI_BINARY" --gui
    else
        print_warning "Binary exists but GUI features not detected."
        print_status "The binary might be CLI-only version."
        print_error "Need to rebuild with GUI features."
        echo ""
        read -p "Would you like to build the GUI version now? (y/n): " -n 1 -r
        echo ""
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            exec "$SCRIPT_DIR/complete-setup.sh"
        else
            exit 1
        fi
    fi
else
    print_warning "GUI binary not found at: $GUI_BINARY"
    print_status "Need to build the GUI version first."
    echo ""
    read -p "Would you like to build the GUI version now? (y/n): " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        exec "$SCRIPT_DIR/complete-setup.sh"
    else
        print_error "Cannot start GUI without building first."
        print_status "Run './complete-setup.sh' to build the GUI version."
        exit 1
    fi
fi
