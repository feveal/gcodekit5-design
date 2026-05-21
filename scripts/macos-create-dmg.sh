#!/bin/bash
# macOS DMG Creation Script
# Creates a distributable DMG file with the application

set -e

VERSION="${1:-1.0.0}"
ARCH="${2:-macos-arm64}"

APP_BUNDLE="GCodeKit.app"
DMG_NAME="gcodekit5-${VERSION}-${ARCH}.dmg"
TEMP_DMG="gcodekit5-temp-${ARCH}.dmg"
VOLUME_NAME="GCodeKit"
MOUNT_POINT="/Volumes/${VOLUME_NAME}"

echo "Creating DMG installer: $DMG_NAME"

# Detach any existing mounts with this volume name
if [ -d "$MOUNT_POINT" ]; then
    echo "Detaching existing volume at $MOUNT_POINT..."
    hdiutil detach "$MOUNT_POINT" -force 2>/dev/null || true
    sleep 1
fi

# Also try to detach any volumes matching the pattern
for vol in /Volumes/GCodeKit*; do
    if [ -d "$vol" ]; then
        echo "Detaching volume: $vol"
        hdiutil detach "$vol" -force 2>/dev/null || true
    fi
done
sleep 1

# Remove any existing DMG files
rm -f "$DMG_NAME" "$TEMP_DMG"

# Create a staging directory with the app and Applications symlink
STAGING_DIR="dmg-staging-${ARCH}"
rm -rf "$STAGING_DIR"
mkdir -p "$STAGING_DIR"
cp -R "$APP_BUNDLE" "$STAGING_DIR/"
ln -s /Applications "$STAGING_DIR/Applications"

# Create the DMG directly from the staging directory
hdiutil create -volname "$VOLUME_NAME" -srcfolder "$STAGING_DIR" -ov -format UDZO -imagekey zlib-level=9 "$DMG_NAME"

# Clean up staging directory
rm -rf "$STAGING_DIR"

echo "DMG created successfully: $DMG_NAME"
ls -lh "$DMG_NAME"
