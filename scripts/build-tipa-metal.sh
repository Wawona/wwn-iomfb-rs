#!/usr/bin/env bash
# TrollStore tipa: Metal encode into IOSurface, IOMFB swap same ID.
# Bump BUILD every reinstall.
set -euo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
OUT="${1:-$HERE/.agent-device/test-artifacts/WawonaIomfbMetal.tipa}"
BUILD="${BUILD:-1}"
VERSION="${VERSION:-26.9.4}"
BUNDLE="com.aspauldingcode.wawona.iomfb.metal"
TOOLS="${VPHONE_TOOLS:-$HOME/.vphone/src/vphone-cli/.tools/bin}"
export PATH="$TOOLS:/usr/bin:/bin:$PATH"

SDK="$(xcrun --sdk iphoneos --show-sdk-path)"
CC="$(xcrun -sdk iphoneos -f clang)"

WORKDIR="$(mktemp -d)"
trap 'rm -rf "$WORKDIR"' EXIT
APP="$WORKDIR/Payload/WawonaIomfbMetal.app"
mkdir -p "$APP"

cat > "$APP/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleIdentifier</key>
  <string>$BUNDLE</string>
  <key>CFBundleName</key>
  <string>WawonaIomfbMetal</string>
  <key>CFBundleDisplayName</key>
  <string>IomfbMetal</string>
  <key>CFBundleExecutable</key>
  <string>WawonaIomfbMetal</string>
  <key>CFBundleShortVersionString</key>
  <string>$VERSION</string>
  <key>CFBundleVersion</key>
  <string>$BUILD</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>MinimumOSVersion</key>
  <string>15.0</string>
  <key>UILaunchStoryboardName</key>
  <string></string>
</dict>
</plist>
EOF

cat > "$WORKDIR/ents.plist" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>get-task-allow</key>
  <true/>
  <key>platform-application</key>
  <true/>
  <key>com.apple.private.security.no-sandbox</key>
  <true/>
  <key>com.apple.private.IOMobileFramebuffer</key>
  <true/>
  <key>com.apple.private.allow-explicit-graphics-priority</key>
  <true/>
  <key>com.apple.IOSurface.IOSurface</key>
  <true/>
  <key>com.apple.security.iokit-user-client-class</key>
  <array>
    <string>IOMobileFramebufferUserClient</string>
    <string>IOSurfaceRootUserClient</string>
  </array>
</dict>
</plist>
EOF

"$CC" -isysroot "$SDK" -arch arm64 -miphoneos-version-min=15.0 \
  -fobjc-arc -O2 \
  -framework CoreFoundation -framework IOSurface -framework Foundation \
  -framework Metal -framework UIKit \
  -o "$APP/WawonaIomfbMetal" \
  "$HERE/examples/tipa-metal/main.m"

if ! command -v ldid >/dev/null; then
  echo "ldid required on PATH" >&2
  exit 1
fi
ldid -S"$WORKDIR/ents.plist" "$APP/WawonaIomfbMetal"

mkdir -p "$(dirname "$OUT")"
(cd "$WORKDIR" && zip -qr "$OUT" Payload)
echo "wrote $OUT build=$BUILD"
echo "install: packages tipa install $OUT --device \"vphone wawona-jb\""
