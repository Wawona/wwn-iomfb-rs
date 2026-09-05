#!/usr/bin/env bash
# Cross-compile the crate-linked live matrix. No ElleKit.
set -euo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
OUT="${1:-$HERE/.agent-device/test-artifacts/iomfb-matrix}"
SDK="$(xcrun --sdk iphoneos --show-sdk-path)"
CC="$(xcrun -sdk iphoneos -f clang)"
export PATH="${VPHONE_TOOLS:-$HOME/.vphone/src/vphone-cli/.tools/bin}:$HOME/.cargo/bin:/usr/bin:/bin:$PATH"
export IPHONEOS_DEPLOYMENT_TARGET="${IPHONEOS_DEPLOYMENT_TARGET:-15.0}"

LIB="$HERE/target/aarch64-apple-ios/release/libiomfb_c.a"
cargo build -p iomfb-c --release --target aarch64-apple-ios --features apple-iomfb

mkdir -p "$(dirname "$OUT")"
"$CC" -isysroot "$SDK" -arch arm64 -miphoneos-version-min=15.0 -O2 \
  -I"$HERE/include" \
  -o "$OUT" "$HERE/examples/jb-cli/matrix.c" \
  "$LIB" -framework Foundation -framework IOSurface -framework Metal -framework CoreFoundation \
  -lc++
if command -v ldid >/dev/null; then
  ldid -S"$HERE/examples/jb-cli/ents.plist" "$OUT"
fi
echo "wrote $OUT"
echo "push: ssh … 'mkdir -p /var/jb/usr/local/bin; dd of=/var/jb/usr/local/bin/iomfb-matrix' < $OUT"
