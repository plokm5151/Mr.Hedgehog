#!/bin/bash
set -e

ICON_SRC="assets/icon.jpg"
ICONSET_DIR="assets/AppIcon.iconset"
ICNS_DEST="assets/AppIcon.icns"

mkdir -p "$ICONSET_DIR"

echo "🎨 Generating iconset from $ICON_SRC..."

# Helper function
resize_icon() {
    size=$1
    name=$2
    sips -z $size $size -s format png "$ICON_SRC" --out "$ICONSET_DIR/$name"
}

resize_icon 16   "icon_16x16.png"
resize_icon 32   "icon_16x16@2x.png"
resize_icon 32   "icon_32x32.png"
resize_icon 64   "icon_32x32@2x.png"
resize_icon 128  "icon_128x128.png"
resize_icon 256  "icon_128x128@2x.png"
resize_icon 256  "icon_256x256.png"
resize_icon 512  "icon_256x256@2x.png"
resize_icon 512  "icon_512x512.png"
resize_icon 1024 "icon_512x512@2x.png"

echo "📦 Converting to .icns..."
iconutil -c icns "$ICONSET_DIR" -o "$ICNS_DEST"

echo "🧹 Cleaning up..."
rm -rf "$ICONSET_DIR"

echo "✅ Generated $ICNS_DEST"
