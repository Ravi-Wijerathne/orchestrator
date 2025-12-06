#!/bin/bash

# File Orchestrator Startup Script
# This script checks dependencies and starts the orchestrator tool

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
echo -e "${BLUE}  File Orchestrator - Dependency Check & Startup${NC}"
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

# Function to check Rust version
check_rust_version() {
    local rust_version=$(rustc --version | awk '{print $2}')
    local major=$(echo $rust_version | cut -d. -f1)
    local minor=$(echo $rust_version | cut -d. -f2)
    
    if [ "$major" -gt 1 ] || ([ "$major" -eq 1 ] && [ "$minor" -ge 70 ]); then
        return 0
    else
        return 1
    fi
}

# Check for Rust
print_status "Checking for Rust installation..."
if command_exists rustc && command_exists cargo; then
    if check_rust_version; then
        RUST_VERSION=$(rustc --version)
        print_success "Rust found: $RUST_VERSION"
    else
        print_error "Rust version is too old. Required: 1.70+"
        print_status "Please update Rust using: rustup update"
        exit 1
    fi
else
    print_error "Rust is not installed!"
    print_status "Please install Rust from: https://rustup.rs"
    print_status "Run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check for Cargo
print_status "Checking for Cargo..."
if command_exists cargo; then
    CARGO_VERSION=$(cargo --version)
    print_success "Cargo found: $CARGO_VERSION"
else
    print_error "Cargo is not installed! (Should come with Rust)"
    exit 1
fi

# Change to project directory
print_status "Navigating to project directory..."
cd "$SCRIPT_DIR"
print_success "Working directory: $SCRIPT_DIR"

# Check if Cargo.toml exists
if [ ! -f "Cargo.toml" ]; then
    print_error "Cargo.toml not found! Are you in the correct directory?"
    exit 1
fi

# Check for configuration file
print_status "Checking for configuration file..."
if [ -f "config.toml" ]; then
    print_success "Configuration file found: config.toml"
elif [ -f "config.example.toml" ]; then
    print_warning "config.toml not found. Found config.example.toml"
    read -p "Would you like to copy config.example.toml to config.toml? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        cp config.example.toml config.toml
        print_success "Created config.toml from config.example.toml"
        print_warning "Please edit config.toml with your settings before continuing"
        read -p "Press Enter when you're ready to continue..."
    else
        print_error "Configuration file is required to run the orchestrator"
        exit 1
    fi
else
    print_warning "No configuration file found (config.toml or config.example.toml)"
    print_warning "The tool may require a configuration file to run properly"
fi

# Check if project is built
print_status "Checking if project is built..."
if [ -f "target/debug/fo" ] || [ -f "target/release/fo" ]; then
    print_success "Binary found in target directory"
else
    print_warning "Binary not found. Building project..."
    print_status "Running: cargo build --release"
    
    if cargo build --release; then
        print_success "Build completed successfully!"
    else
        print_error "Build failed! Please check the error messages above."
        exit 1
    fi
fi

# Check system dependencies (optional but good to have)
print_status "Checking optional system dependencies..."

# Check for git (good to have for version control)
if command_exists git; then
    print_success "git found: $(git --version)"
else
    print_warning "git not found (optional)"
fi

# Summary
echo ""
echo -e "${GREEN}==================================================${NC}"
echo -e "${GREEN}  All dependencies checked successfully!${NC}"
echo -e "${GREEN}==================================================${NC}"
echo ""

# Ask user which mode to run
print_status "Select running mode:"
echo "  1) Install globally (recommended - use 'fo' command anywhere)"
echo "  2) Run in debug mode (faster compilation, includes debug info)"
echo "  3) Run in release mode (optimized, recommended for production)"
echo "  4) Build only (don't run)"
echo "  5) Show help"
echo ""
read -p "Enter your choice (1-5): " -n 1 -r
echo ""

case $REPLY in
    1)
        print_status "Installing globally to ~/.cargo/bin/..."
        if cargo install --path .; then
            print_success "Successfully installed! You can now use 'fo' command from anywhere!"
            echo ""
            print_status "Try running: fo --help"
            
            # Check if ~/.cargo/bin is in PATH
            if [[ ":$PATH:" != *":$HOME/.cargo/bin:"* ]]; then
                print_warning "~/.cargo/bin is not in your PATH!"
                print_status "Add this line to your ~/.bashrc or ~/.zshrc:"
                echo ""
                echo "  export PATH=\"\$HOME/.cargo/bin:\$PATH\""
                echo ""
                print_status "Then run: source ~/.bashrc (or source ~/.zshrc)"
            fi
        else
            print_error "Installation failed! Please check the error messages above."
            exit 1
        fi
        ;;
    2)
        print_status "Building and running in debug mode..."
        cargo run -- "$@"
        ;;
    3)
        print_status "Building and running in release mode..."
        cargo run --release -- "$@"
        ;;
    4)
        print_status "Building in release mode..."
        cargo build --release
        print_success "Build complete! Binary location: target/release/fo"
        ;;
    5)
        print_status "Showing help..."
        if [ -f "target/release/fo" ]; then
            ./target/release/fo --help
        elif [ -f "target/debug/fo" ]; then
            ./target/debug/fo --help
        else
            cargo run -- --help
        fi
        ;;
    *)
        print_error "Invalid choice. Exiting."
        exit 1
        ;;
esac

echo ""
print_success "Done!"
