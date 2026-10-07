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
  <key>CFBundleURLTypes</key>
  <array>
    <dict>
      <key>CFBundleURLName</key>
      <string>$BUNDLE</string>
      <key>CFBundleURLSchemes</key>
      <array>
        <string>wawona-iomfb-metal</string>
      </array>
    </dict>
  </array>
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
    <string>IOGPUDeviceUserClient</string>
    <string>AGXDeviceUserClient</string>
    <string>IOGPUMemoryInfoUserClient</string>
  </array>
</dict>
</plist>
EOF

SWIFT=(xcrun -sdk iphoneos swiftc)
METAL_PRESENT="$HERE/examples/tipa-metal/tipa_metal_present.c"
METAL_APP="$HERE/examples/tipa-metal/TipaMetalApp.swift"
"$CC" -isysroot "$SDK" -arch arm64 -miphoneos-version-min=15.0 \
  -x objective-c -fobjc-arc -O2 -c "$METAL_PRESENT" -o "$WORKDIR/tipa_metal_present.o"
"${SWIFT[@]}" -sdk "$SDK" -target arm64-apple-ios15.0 -O \
  -import-objc-header "$HERE/examples/tipa-metal/tipa_metal_present.h" \
  -c "$METAL_APP" -o "$WORKDIR/TipaMetalApp.o"
"$CC" -isysroot "$SDK" -arch arm64 -miphoneos-version-min=15.0 \
  -fobjc-arc -O2 \
  -framework CoreFoundation -framework IOSurface -framework Foundation \
  -framework Metal -framework UIKit -framework IOKit \
  "$WORKDIR/tipa_metal_present.o" "$WORKDIR/TipaMetalApp.o" \
  -o "$APP/WawonaIomfbMetal"

if ! command -v ldid >/dev/null; then
  echo "ldid required on PATH" >&2
  exit 1
fi
ldid -S"$WORKDIR/ents.plist" "$APP/WawonaIomfbMetal"

mkdir -p "$(dirname "$OUT")"
(cd "$WORKDIR" && zip -qr "$OUT" Payload)
echo "wrote $OUT build=$BUILD"
echo "install: packages tipa install $OUT --device \"vphone wawona-jb\""
