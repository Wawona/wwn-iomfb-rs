#!/usr/bin/env bash
# Cross-compile the Sileo-path CLI. No ElleKit.
set -euo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
OUT="${1:-$HERE/.agent-device/test-artifacts/iomfb-cli}"
SDK="$(xcrun --sdk iphoneos --show-sdk-path)"
CC="$(xcrun -sdk iphoneos -f clang)"
mkdir -p "$(dirname "$OUT")"
"$CC" -isysroot "$SDK" -arch arm64 -miphoneos-version-min=15.0 -O2 \
  -o "$OUT" "$HERE/examples/jb-cli/main.c"
if command -v ldid >/dev/null; then
  ldid -S"$HERE/examples/jb-cli/ents.plist" "$OUT"
fi
echo "wrote $OUT"
echo "push: ssh … 'mkdir -p /var/jb/usr/local/bin; dd of=/var/jb/usr/local/bin/iomfb-cli' < $OUT"
