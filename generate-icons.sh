#!/bin/bash

# Generate placeholder PNG icons using ImageMagick

echo "Generating placeholder icons..."

# Check if ImageMagick is installed
if ! command -v convert &> /dev/null; then
    echo "ImageMagick not found. Installing..."
    sudo apt-get update && sudo apt-get install -y imagemagick
fi

# Create a base icon with ImageMagick
convert -size 512x512 xc:"#2196f3" \
    -gravity center \
    -pointsize 300 \
    -font "DejaVu-Sans" \
    -fill white \
    -annotate +0+0 "📁" \
    icons/icon.png 2>/dev/null || \
convert -size 512x512 xc:"#2196f3" \
    -gravity center \
    -pointsize 200 \
    -font "DejaVu-Sans" \
    -fill white \
    -annotate +0+0 "FO" \
    icons/icon.png

# Generate different sizes
convert icons/icon.png -resize 32x32 icons/32x32.png
convert icons/icon.png -resize 128x128 icons/128x128.png
convert icons/icon.png -resize 128x128 icons/128x128@2x.png
convert icons/icon.png -resize 256x256 icons/icon.ico

echo "✓ Icons generated in icons/ directory"
echo "You can replace icons/icon.png with your own 512x512 icon and re-run this script"
