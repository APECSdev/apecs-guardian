#!/usr/bin/env bash
# packaging/appimage.sh
#
# Builds apecs-linux-x86_64.AppImage from the release Linux binary.
#
# Usage:   packaging/appimage.sh [path-to-linux-binary]
# Default: dist/apecs-linux-x86_64
#
# Requires: appimagetool on PATH (the CI runner has it at /usr/local/bin),
#           and FUSE (or appimagetool's --appimage-extract-and-run fallback).
# Output:   dist-appimage/apecs-linux-x86_64.AppImage

set -euo pipefail

BIN="${1:-dist/apecs-linux-x86_64}"
OUT_DIR="dist-appimage"
APP_DIR="packaging/AppDir"

[ -f "$BIN" ] || { echo "error: binary not found: $BIN" >&2; exit 1; }
command -v appimagetool >/dev/null || { echo "error: appimagetool not on PATH" >&2; exit 1; }

# 1. Stage the AppDir.
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/usr/bin" "$OUT_DIR"
cp "$BIN" "$APP_DIR/usr/bin/apecs"
chmod +x "$APP_DIR/usr/bin/apecs"

# 2. AppRun entry point.
cat > "$APP_DIR/AppRun" <<'APPRUN'
#!/bin/sh
exec "$(dirname "$0")/usr/bin/apecs" "$@"
APPRUN
chmod +x "$APP_DIR/AppRun"

# 3. Desktop file + icon (icon is committed at packaging/apecs.png).
cat > "$APP_DIR/apecs.desktop" <<'DESKTOP'
[Desktop Entry]
Type=Application
Name=APECS Guardian
Comment=Crypto systems engineering, handled.
Exec=apecs
Icon=apecs
Categories=Utility;
Terminal=true
DESKTOP
cp packaging/apecs.png "$APP_DIR/apecs.png"

# 4. Build the AppImage.
export ARCH=x86_64
appimagetool "$APP_DIR" "$OUT_DIR/apecs-linux-x86_64.AppImage"

# 5. Leave the tree clean for repeatable CI runs.
rm -rf "$APP_DIR"

ls -la "$OUT_DIR/apecs-linux-x86_64.AppImage"
