#!/usr/bin/env bash
# TrollStore tipa: heavy Metal shader bench through iomfb-c.
# MIT. Bump BUILD every reinstall.
set -euo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
OUT="${1:-$HERE/.agent-device/test-artifacts/WawonaIomfbBench.tipa}"
BUILD="${BUILD:-1}"
VERSION="${VERSION:-26.9.4}"
BUNDLE="com.aspauldingcode.wawona.iomfb.bench"
TOOLS="${VPHONE_TOOLS:-$HOME/.vphone/src/vphone-cli/.tools/bin}"
export PATH="$TOOLS:$HOME/.cargo/bin:/usr/bin:/bin:$PATH"
export IPHONEOS_DEPLOYMENT_TARGET="${IPHONEOS_DEPLOYMENT_TARGET:-15.0}"

SDK="$(xcrun --sdk iphoneos --show-sdk-path)"
CC="$(xcrun -sdk iphoneos -f clang)"
METAL=(xcrun -sdk iphoneos metal)
METALLIB=(xcrun -sdk iphoneos metallib)

LIB="$HERE/target/aarch64-apple-ios/release/libiomfb_c.a"
if [[ ! -f "$LIB" ]]; then
  cargo build -p iomfb-c --release --target aarch64-apple-ios --features apple-iomfb
fi

WORKDIR="$(mktemp -d)"
trap 'rm -rf "$WORKDIR"' EXIT
APP="$WORKDIR/Payload/WawonaIomfbBench.app"
mkdir -p "$APP"

"${METAL[@]}" -std=ios-metal2.4 \
  -c "$HERE/examples/tipa-bench/Shaders.metal" -o "$WORKDIR/shaders.air"
"${METALLIB[@]}" "$WORKDIR/shaders.air" -o "$APP/default.metallib"

cat > "$APP/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleIdentifier</key>
  <string>$BUNDLE</string>
  <key>CFBundleName</key>
  <string>WawonaIomfbBench</string>
  <key>CFBundleDisplayName</key>
  <string>IomfbBench</string>
  <key>CFBundleExecutable</key>
  <string>WawonaIomfbBench</string>
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
        <string>wawona-iomfb-bench</string>
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

"$CC" -isysroot "$SDK" -arch arm64 -miphoneos-version-min=15.0 \
  -fobjc-arc -O2 \
  -I"$HERE/include" \
  -framework CoreFoundation -framework IOSurface -framework Foundation \
  -framework Metal -framework UIKit -framework QuartzCore -framework IOKit \
  -Wl,-force_load,"$LIB" \
  -o "$APP/WawonaIomfbBench" \
  "$HERE/examples/tipa-bench/main.m"

if ! command -v ldid >/dev/null; then
  echo "ldid required on PATH" >&2
  exit 1
fi
ldid -S"$WORKDIR/ents.plist" "$APP/WawonaIomfbBench"

mkdir -p "$(dirname "$OUT")"
(cd "$WORKDIR" && zip -qr "$OUT" Payload)
echo "wrote $OUT build=$BUILD"
echo "install: packages tipa install $OUT --device \"vphone wawona-jb\""
echo "open: uiopen wawona-iomfb-bench://"
