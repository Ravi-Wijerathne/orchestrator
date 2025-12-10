#!/bin/bash

# File Orchestrator - Complete Setup and Demo Script
# This script sets up the GUI and demonstrates all features

set -e

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║          File Orchestrator - Complete Setup & Demo            ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_info() {
    echo -e "${BLUE}[ℹ]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[⚠]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
}

# Check prerequisites
echo "📋 Checking Prerequisites..."
echo ""

# Check Rust
if command -v cargo &> /dev/null; then
    print_status "Rust: $(rustc --version)"
else
    print_error "Rust is not installed!"
    echo "Install from: https://rustup.rs"
    exit 1
fi

# Check Node.js
if command -v node &> /dev/null; then
    print_status "Node.js: $(node --version)"
else
    print_error "Node.js is not installed!"
    echo "Install with: curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -"
    echo "              sudo apt-get install -y nodejs"
    exit 1
fi

# Check npm
if command -v npm &> /dev/null; then
    print_status "npm: $(npm --version)"
else
    print_error "npm is not installed!"
    exit 1
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo ""

# Step 1: Install Frontend Dependencies
echo "📦 Step 1: Installing Frontend Dependencies..."
echo ""

if [ -d "ui" ]; then
    cd ui
    
    if [ ! -d "node_modules" ]; then
        print_info "Installing npm packages (this may take a few minutes)..."
        npm install
        print_status "Frontend dependencies installed"
    else
        print_status "Frontend dependencies already installed"
    fi
    
    cd ..
else
    print_error "ui/ directory not found!"
    exit 1
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo ""

# Step 2: Generate Icons
echo "🎨 Step 2: Generating Application Icons..."
echo ""

if [ ! -d "icons" ]; then
    mkdir -p icons
fi

# Check for ImageMagick
if command -v convert &> /dev/null; then
    print_info "Generating icons with ImageMagick..."
    
    # Create base icon
    convert -size 512x512 xc:"#2196f3" \
        -gravity center \
        -pointsize 200 \
        -font "DejaVu-Sans" \
        -fill white \
        -annotate +0+0 "FO" \
        icons/icon.png 2>/dev/null || {
        convert -size 512x512 xc:"#2196f3" icons/icon.png
    }
    
    # Generate other sizes
    convert icons/icon.png -resize 32x32 icons/32x32.png 2>/dev/null
    convert icons/icon.png -resize 128x128 icons/128x128.png 2>/dev/null
    convert icons/icon.png -resize 256x256 icons/128x128@2x.png 2>/dev/null
    
    print_status "Icons generated"
else
    print_warning "ImageMagick not found, using placeholder icons"
    print_info "Install ImageMagick for better icons: sudo apt-get install imagemagick"
    
    # Create placeholder SVG
    cat > icons/icon.svg << 'EOF'
<svg width="512" height="512" xmlns="http://www.w3.org/2000/svg">
  <rect width="512" height="512" fill="#2196f3"/>
  <text x="256" y="310" font-family="Arial" font-size="180" fill="white" text-anchor="middle" font-weight="bold">FO</text>
</svg>
EOF
    print_status "Placeholder icon created"
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo ""

# Step 3: Build the Application
echo "🔨 Step 3: Building File Orchestrator with GUI..."
echo ""

print_info "This will compile the Rust backend and bundle the frontend"
print_info "First build may take several minutes..."
echo ""

cargo build --features gui --release

if [ $? -eq 0 ]; then
    print_status "Build completed successfully!"
else
    print_error "Build failed!"
    exit 1
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo ""

# Step 4: Create Demo Configuration
echo "⚙️  Step 4: Creating Demo Configuration..."
echo ""

if [ ! -f "config.toml" ]; then
    print_info "Creating sample configuration..."
    
    cat > config.toml << 'EOF'
source_dir = "/tmp/file-orchestrator-demo"

[file_categories]
documents = ["pdf", "doc", "docx", "txt", "xlsx", "pptx", "odt"]
images = ["jpg", "jpeg", "png", "gif", "bmp", "webp", "svg", "tiff"]
videos = ["mp4", "avi", "mov", "mkv", "flv", "wmv", "webm", "m4v"]
audio = ["mp3", "wav", "flac", "aac", "ogg", "m4a", "wma"]
archives = ["zip", "rar", "7z", "tar", "gz", "bz2"]

[[drives]]
label = "Demo_Documents"
category = "documents"
mount_point = "/tmp/demo-drive-docs"

[[drives]]
label = "Demo_Images"
category = "images"
mount_point = "/tmp/demo-drive-images"
EOF

    print_status "Demo configuration created"
    
    # Create demo directories
    mkdir -p /tmp/file-orchestrator-demo
    mkdir -p /tmp/demo-drive-docs
    mkdir -p /tmp/demo-drive-images
    
    # Create some demo files
    echo "Demo document" > /tmp/file-orchestrator-demo/demo.txt
    echo "Sample PDF content" > /tmp/file-orchestrator-demo/sample.pdf
    
    print_status "Demo directories and files created"
else
    print_status "Configuration already exists"
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo ""

# Step 5: Success Message
echo "✅ Setup Complete!"
echo ""
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                  🎉 READY TO RUN! 🎉                           ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

print_info "The File Orchestrator GUI is now ready!"
echo ""

echo "🚀 To Start the Application:"
echo ""
echo "   ./target/release/fo --gui"
echo ""

echo "📖 Available Documentation:"
echo "   • GUI_README.md           - Complete GUI documentation"
echo "   • GUI_SETUP_GUIDE.md      - Detailed setup instructions"
echo "   • GUI_QUICK_REFERENCE.md  - Quick reference card"
echo "   • ARCHITECTURE_GUI.md     - Architecture diagrams"
echo ""

echo "🎯 Quick Feature Tour:"
echo "   1. Dashboard (/)          - Real-time statistics and controls"
echo "   2. Drives (/drives)       - Manage USB drives"
echo "   3. Pending (/pending)     - View queued files"
echo "   4. History (/history)     - Sync timeline"
echo "   5. Statistics (/stats)    - Interactive charts"
echo "   6. Settings (/config)     - Visual configuration"
echo "   7. System Tray            - Background operations"
echo ""

echo "🔧 Development Mode (with hot-reload):"
echo "   Terminal 1: cd ui && npm run dev"
echo "   Terminal 2: cargo run --features gui -- --gui"
echo ""

echo "📚 CLI Mode Still Available:"
echo "   cargo run --release -- run"
echo "   cargo run --release -- status"
echo "   cargo run --release -- list-drives"
echo ""

echo "════════════════════════════════════════════════════════════════"
echo ""

# Ask if user wants to start the app now
read -p "Would you like to start the GUI now? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo ""
    print_info "Starting File Orchestrator GUI..."
    echo ""
    ./target/release/fo --gui
fi

echo ""
echo "Thank you for using File Orchestrator! 🚀"
echo ""
