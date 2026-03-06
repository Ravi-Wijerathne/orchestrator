#!/bin/bash

# File Orchestrator - Dependency Checker
# Verifies all prerequisites before installation

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   🔍 Dependency & Prerequisites Check  ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════╝${NC}"
echo ""

ERRORS=0
WARNINGS=0

# Function to check command exists
check_command() {
    local cmd=$1
    local name=$2
    local install_hint=$3
    local required=$4
    
    if command -v "$cmd" &> /dev/null; then
        local version=$($cmd --version 2>&1 | head -n 1)
        echo -e "${GREEN}✓${NC} ${BOLD}$name${NC} - Found"
        echo -e "  ${BLUE}→${NC} $version"
        return 0
    else
        if [ "$required" = "true" ]; then
            echo -e "${RED}✗${NC} ${BOLD}$name${NC} - ${RED}NOT FOUND (REQUIRED)${NC}"
            echo -e "  ${YELLOW}→${NC} Install: $install_hint"
            ((ERRORS++))
            return 1
        else
            echo -e "${YELLOW}⚠${NC} ${BOLD}$name${NC} - ${YELLOW}NOT FOUND (OPTIONAL)${NC}"
            echo -e "  ${BLUE}→${NC} Install: $install_hint"
            ((WARNINGS++))
            return 2
        fi
    fi
}

# Function to check Rust toolchain components
check_rust_toolchain() {
    if ! command -v rustc &> /dev/null; then
        return 1
    fi
    
    echo ""
    echo -e "${BLUE}📦 Checking Rust Toolchain...${NC}"
    
    # Check cargo
    if command -v cargo &> /dev/null; then
        echo -e "${GREEN}✓${NC} Cargo - $(cargo --version)"
    else
        echo -e "${RED}✗${NC} Cargo - NOT FOUND"
        ((ERRORS++))
    fi
    
    # Check rustup
    if command -v rustup &> /dev/null; then
        echo -e "${GREEN}✓${NC} Rustup - $(rustup --version | head -n 1)"
        
        # Check default toolchain
        local toolchain=$(rustup show active-toolchain 2>&1)
        echo -e "  ${BLUE}→${NC} Active toolchain: $toolchain"
    else
        echo -e "${YELLOW}⚠${NC} Rustup - NOT FOUND (recommended for updates)"
    fi
}

# Function to check system libraries
check_system_libs() {
    echo ""
    echo -e "${BLUE}📚 Checking System Libraries...${NC}"
    
    local libs_ok=true
    
    # Check for GUI dependencies (Linux)
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Check pkg-config
        if command -v pkg-config &> /dev/null; then
            echo -e "${GREEN}✓${NC} pkg-config - Found"
            
            # Check for required GUI libraries
            local gui_libs=("x11" "xext" "xcursor" "xrandr" "xi")
            local missing_libs=()
            
            for lib in "${gui_libs[@]}"; do
                if pkg-config --exists "$lib" 2>/dev/null; then
                    echo -e "${GREEN}✓${NC} lib$lib - Found"
                else
                    echo -e "${RED}✗${NC} lib$lib - NOT FOUND"
                    missing_libs+=("lib$lib-dev")
                    libs_ok=false
                fi
            done
            
            if [ ${#missing_libs[@]} -gt 0 ]; then
                echo -e ""
                echo -e "${YELLOW}Missing GUI libraries. Install with:${NC}"
                echo -e "  ${BOLD}sudo apt install ${missing_libs[*]}${NC}"
                ((ERRORS++))
            fi
        else
            echo -e "${YELLOW}⚠${NC} pkg-config - NOT FOUND (cannot verify libraries)"
            ((WARNINGS++))
        fi
        
        # Check for fontconfig (needed for text rendering)
        if pkg-config --exists fontconfig 2>/dev/null; then
            echo -e "${GREEN}✓${NC} fontconfig - Found"
        else
            echo -e "${RED}✗${NC} fontconfig - NOT FOUND"
            echo -e "  ${YELLOW}→${NC} Install: sudo apt install libfontconfig1-dev"
            ((ERRORS++))
            libs_ok=false
        fi
        
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo -e "${GREEN}✓${NC} macOS detected - GUI libraries built-in"
    fi
}

# Function to check disk space
check_disk_space() {
    echo ""
    echo -e "${BLUE}💾 Checking Disk Space...${NC}"
    
    local available=$(df . | awk 'NR==2 {print $4}')
    local available_mb=$((available / 1024))
    
    if [ "$available_mb" -gt 1000 ]; then
        echo -e "${GREEN}✓${NC} Available space: ${available_mb} MB"
    elif [ "$available_mb" -gt 500 ]; then
        echo -e "${YELLOW}⚠${NC} Available space: ${available_mb} MB (low)"
        ((WARNINGS++))
    else
        echo -e "${RED}✗${NC} Available space: ${available_mb} MB (insufficient)"
        echo -e "  ${YELLOW}→${NC} Recommended: At least 500 MB free"
        ((ERRORS++))
    fi
}

# Function to check USB drive detection capability
check_usb_support() {
    echo ""
    echo -e "${BLUE}🔌 Checking USB Support...${NC}"
    
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if [ -d "/dev/disk/by-id" ]; then
            echo -e "${GREEN}✓${NC} USB device detection available"
        else
            echo -e "${YELLOW}⚠${NC} Cannot access /dev/disk/by-id"
            echo -e "  ${BLUE}→${NC} May need elevated permissions for drive detection"
            ((WARNINGS++))
        fi
        
        # Check if user is in plugdev group (for USB access without sudo)
        if groups | grep -q plugdev; then
            echo -e "${GREEN}✓${NC} User in 'plugdev' group - can access USB devices"
        else
            echo -e "${YELLOW}⚠${NC} User not in 'plugdev' group"
            echo -e "  ${BLUE}→${NC} Add with: sudo usermod -aG plugdev $USER"
            echo -e "  ${BLUE}→${NC} Then logout/login for changes to take effect"
            ((WARNINGS++))
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        if [ -d "/Volumes" ]; then
            echo -e "${GREEN}✓${NC} USB mount point available (/Volumes)"
        else
            echo -e "${RED}✗${NC} Cannot access /Volumes"
            ((ERRORS++))
        fi
    fi
}

echo -e "${BOLD}Checking Required Dependencies...${NC}"
echo ""

# Required dependencies
check_command "rustc" "Rust Compiler" "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh" "true"
check_command "cargo" "Cargo (Rust Package Manager)" "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh" "true"

# Optional but recommended
echo ""
echo -e "${BOLD}Checking Optional Dependencies...${NC}"
echo ""
check_command "git" "Git" "sudo apt install git" "false"
check_command "rustup" "Rustup (Rust Toolchain Manager)" "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh" "false"

# Additional checks
if command -v rustc &> /dev/null; then
    check_rust_toolchain
fi

check_system_libs
check_disk_space
check_usb_support

# Summary
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BOLD}Summary:${NC}"
echo ""

if [ $ERRORS -eq 0 ] && [ $WARNINGS -eq 0 ]; then
    echo -e "${GREEN}✨ All checks passed! You're ready to install.${NC}"
    echo ""
    echo -e "${BLUE}Next step:${NC}"
    echo -e "  Run: ${BOLD}../scripts/start.sh${NC}"
    exit 0
elif [ $ERRORS -eq 0 ]; then
    echo -e "${YELLOW}⚠️  ${WARNINGS} warning(s) found${NC}"
    echo -e "${GREEN}✓ All required dependencies are installed${NC}"
    echo ""
    echo -e "${BLUE}You can proceed, but some features may be limited.${NC}"
    echo ""
    echo -e "${BLUE}Next step:${NC}"
    echo -e "  Run: ${BOLD}../scripts/start.sh${NC}"
    exit 0
else
    echo -e "${RED}✗ ${ERRORS} error(s) found${NC}"
    if [ $WARNINGS -gt 0 ]; then
        echo -e "${YELLOW}⚠️  ${WARNINGS} warning(s) found${NC}"
    fi
    echo ""
    echo -e "${RED}Please install required dependencies before proceeding.${NC}"
    exit 1
fi
