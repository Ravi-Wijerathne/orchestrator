#!/usr/bin/env python3

"""
File Orchestrator - Smart Launcher
Automatically detects first run and guides setup
"""

import os
import sys
import subprocess
import re
from pathlib import Path

# ANSI color codes
RED = '\033[0;31m'
GREEN = '\033[0;32m'
YELLOW = '\033[1;33m'
BLUE = '\033[0;34m'
NC = '\033[0m'  # No Color

# Get project root directory (parent of scripts/)
SCRIPT_DIR = Path(__file__).parent.resolve()
PROJECT_DIR = SCRIPT_DIR.parent
CONFIG_FILE = PROJECT_DIR / "config.toml"
BINARY = PROJECT_DIR / "target" / "release" / "fo"

# Adjust for Windows
if sys.platform == "win32":
    BINARY = PROJECT_DIR / "target" / "release" / "fo.exe"

def print_header():
    """Print the welcome header"""
    print(f"{BLUE}╔════════════════════════════════════════════╗{NC}")
    print(f"{BLUE}║     🗂️  File Orchestrator - GUI      ║{NC}")
    print(f"{BLUE}╚════════════════════════════════════════════╝{NC}")
    print()

def build_binary():
    """Build the binary with GUI support"""
    print(f"{BLUE}📦 Binary not found, building now...{NC}")
    print(f"{YELLOW}Building File Orchestrator with GUI support...{NC}")
    print()
    
    try:
        subprocess.run(
            ["cargo", "build", "--features", "gui", "--release"],
            cwd=PROJECT_DIR,
            check=True
        )
        print()
        
        if not BINARY.exists():
            print(f"{RED}❌ Build failed!{NC}")
            sys.exit(1)
        
        print(f"{GREEN}✓ Build successful!{NC}")
        print()
    except subprocess.CalledProcessError:
        print(f"{RED}❌ Build failed!{NC}")
        sys.exit(1)
    except FileNotFoundError:
        print(f"{RED}❌ Cargo not found! Make sure Rust is installed.{NC}")
        sys.exit(1)

def get_storage_path():
    """Prompt user for storage path"""
    home_dir = Path.home()
    
    while True:
        print(f"{GREEN}Enter the path to your main storage folder:{NC}")
        print(f"{BLUE}(This is where File Orchestrator will watch for new files){NC}")
        print(f"{YELLOW}Example: {home_dir}/Documents/MyStorage{NC}")
        
        storage_path = input("Path: ").strip()
        
        # Expand ~ to home directory
        if storage_path.startswith("~"):
            storage_path = storage_path.replace("~", str(home_dir), 1)
        
        if not storage_path:
            print(f"{RED}❌ Path cannot be empty!{NC}")
            print()
            continue
        
        storage_path = Path(storage_path)
        
        # Check if path exists
        if storage_path.is_dir():
            print(f"{GREEN}✓ Path exists!{NC}")
            return str(storage_path)
        else:
            print()
            print(f"{YELLOW}⚠️  Directory doesn't exist: {storage_path}{NC}")
            create_dir = input("Create it now? (y/n): ").strip().lower()
            
            if create_dir in ('y', 'yes'):
                try:
                    storage_path.mkdir(parents=True, exist_ok=True)
                    print(f"{GREEN}✓ Directory created successfully!{NC}")
                    return str(storage_path)
                except OSError:
                    print(f"{RED}❌ Failed to create directory!{NC}")
                    print()
                    continue
            else:
                print()
                continue

def init_config(storage_path):
    """Initialize and configure the config file"""
    print()
    print(f"{BLUE}Creating configuration file...{NC}")
    
    try:
        # Initialize config using the binary
        subprocess.run(
            [str(BINARY), "init"],
            cwd=PROJECT_DIR,
            check=True
        )
        
        # Update the source path in config
        if CONFIG_FILE.exists():
            config_content = CONFIG_FILE.read_text()
            # Replace backslashes with forward slashes for TOML compatibility on Windows
            storage_path_escaped = storage_path.replace("\\", "/")
            # Replace the path in the config file
            config_content = re.sub(
                r'path = ".*?"',
                f'path = "{storage_path_escaped}"',
                config_content
            )
            CONFIG_FILE.write_text(config_content)
        
        print(f"{GREEN}✓ Configuration created!{NC}")
        print()
    except subprocess.CalledProcessError as e:
        print(f"{RED}❌ Failed to initialize configuration!{NC}")
        sys.exit(1)
    except Exception as e:
        print(f"{RED}❌ Error: {e}{NC}")
        sys.exit(1)

def print_setup_complete(storage_path):
    """Print setup completion message"""
    print(f"{GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{NC}")
    print(f"{GREEN}✨ Setup complete!{NC}")
    print()
    print(f"{BLUE}📝 Next steps:{NC}")
    print(f"   1. GUI will open automatically")
    print(f"   2. Go to 'Drive Manager' to register your USB drives")
    print(f"   3. Add files to: {YELLOW}{storage_path}{NC}")
    print(f"   4. They will sync automatically!")
    print(f"{GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{NC}")
    print()
    input("Press Enter to launch GUI...")

def launch_gui():
    """Launch the GUI application"""
    print()
    print(f"{BLUE}🚀 Starting File Orchestrator GUI...{NC}")
    print()
    
    try:
        subprocess.run(
            [str(BINARY), "--gui"],
            cwd=PROJECT_DIR,
            check=False
        )
    except Exception as e:
        print(f"{RED}❌ Failed to launch GUI: {e}{NC}")
        sys.exit(1)

def main():
    """Main function"""
    print_header()
    
    # Check if binary exists
    if not BINARY.exists():
        build_binary()
    
    # Check if config exists
    if not CONFIG_FILE.exists():
        print(f"{YELLOW}🎯 First time setup detected!{NC}")
        print(f"{BLUE}Let's configure your storage folder...{NC}")
        print()
        
        storage_path = get_storage_path()
        init_config(storage_path)
        print_setup_complete(storage_path)
    else:
        print(f"{GREEN}✓ Configuration found{NC}")
    
    # Launch GUI
    launch_gui()

if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print()
        print(f"{YELLOW}⚠️  Cancelled by user{NC}")
        sys.exit(130)
