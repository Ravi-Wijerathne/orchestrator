#!/bin/bash

# File Orchestrator GUI Setup Script

set -e

echo "🚀 File Orchestrator GUI Setup"
echo "================================"
echo ""

# Check for Node.js
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is not installed!"
    echo "Please install Node.js first:"
    echo "  curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -"
    echo "  sudo apt-get install -y nodejs"
    exit 1
fi

echo "✓ Node.js found: $(node --version)"
echo "✓ npm found: $(npm --version)"
echo ""

# Install frontend dependencies
echo "📦 Installing frontend dependencies..."
cd ui
npm install
cd ..
echo "✓ Frontend dependencies installed"
echo ""

# Create placeholder icons if they don't exist
echo "🎨 Setting up icons..."
mkdir -p icons

# Create a simple SVG icon as placeholder
cat > icons/icon.svg << 'EOF'
<svg width="512" height="512" xmlns="http://www.w3.org/2000/svg">
  <rect width="512" height="512" fill="#2196f3"/>
  <text x="256" y="280" font-family="Arial" font-size="200" fill="white" text-anchor="middle">📁</text>
</svg>
EOF

echo "✓ Created placeholder icon (icons/icon.svg)"
echo ""
echo "⚠️  Note: For production, replace icons/icon.svg with your custom icon"
echo "   and regenerate with: npx @tauri-apps/cli icon icons/icon.png"
echo ""

echo "✅ Setup complete!"
echo ""
echo "To run in development mode:"
echo "  1. Terminal 1: cd ui && npm run dev"
echo "  2. Terminal 2: cargo run --features gui --bin fo -- --gui"
echo ""
echo "Or build for production:"
echo "  cargo build --features gui --release"
echo ""
