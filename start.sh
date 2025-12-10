#!/bin/bash

# File Orchestrator - Smart Launcher
# Automatically detects first run and guides setup

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_FILE="$SCRIPT_DIR/config.toml"
BINARY="$SCRIPT_DIR/target/release/fo"

echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║     🗂️  File Orchestrator - GUI      ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════╝${NC}"
echo ""

# Check if binary exists
if [ ! -f "$BINARY" ]; then
    echo -e "${RED}❌ Binary not found!${NC}"
    echo -e "${YELLOW}Building File Orchestrator with GUI support...${NC}"
    echo ""
    cargo build --features gui --release
    echo ""
    if [ ! -f "$BINARY" ]; then
        echo -e "${RED}❌ Build failed!${NC}"
        exit 1
    fi
    echo -e "${GREEN}✓ Build successful!${NC}"
    echo ""
fi

# Check if config exists
if [ ! -f "$CONFIG_FILE" ]; then
    echo -e "${YELLOW}🎯 First time setup detected!${NC}"
    echo -e "${BLUE}Let's configure your storage folder...${NC}"
    echo ""
    
    # Prompt for storage path
    while true; do
        echo -e "${GREEN}Enter the path to your main storage folder:${NC}"
        echo -e "${BLUE}(This is where File Orchestrator will watch for new files)${NC}"
        echo -e "${YELLOW}Example: /home/$USER/Documents/MyStorage${NC}"
        read -p "Path: " storage_path
        
        # Expand ~ to home directory
        storage_path="${storage_path/#\~/$HOME}"
        
        if [ -z "$storage_path" ]; then
            echo -e "${RED}❌ Path cannot be empty!${NC}"
            echo ""
            continue
        fi
        
        # Check if path exists
        if [ -d "$storage_path" ]; then
            echo -e "${GREEN}✓ Path exists!${NC}"
            break
        else
            echo ""
            echo -e "${YELLOW}⚠️  Directory doesn't exist: $storage_path${NC}"
            read -p "Create it now? (y/n): " create_dir
            
            if [[ "$create_dir" =~ ^[Yy]$ ]]; then
                mkdir -p "$storage_path"
                if [ -d "$storage_path" ]; then
                    echo -e "${GREEN}✓ Directory created successfully!${NC}"
                    break
                else
                    echo -e "${RED}❌ Failed to create directory!${NC}"
                    echo ""
                    continue
                fi
            else
                echo ""
                continue
            fi
        fi
    done
    
    echo ""
    echo -e "${BLUE}Creating configuration file...${NC}"
    
    # Initialize config
    "$BINARY" init
    
    # Update the source path in config
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        sed -i '' "s|path = \".*\"|path = \"$storage_path\"|" "$CONFIG_FILE"
    else
        # Linux
        sed -i "s|path = \".*\"|path = \"$storage_path\"|" "$CONFIG_FILE"
    fi
    
    echo -e "${GREEN}✓ Configuration created!${NC}"
    echo ""
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}✨ Setup complete!${NC}"
    echo ""
    echo -e "${BLUE}📝 Next steps:${NC}"
    echo -e "   1. GUI will open automatically"
    echo -e "   2. Go to 'Drive Manager' to register your USB drives"
    echo -e "   3. Add files to: ${YELLOW}$storage_path${NC}"
    echo -e "   4. They will sync automatically!"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    read -p "Press Enter to launch GUI..."
else
    echo -e "${GREEN}✓ Configuration found${NC}"
fi

# Launch GUI
echo ""
echo -e "${BLUE}🚀 Starting File Orchestrator GUI...${NC}"
echo ""

cd "$SCRIPT_DIR"
exec "$BINARY" --gui
