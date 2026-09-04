#!/usr/bin/env bash
# Import the guest-class IOMFB Mach-O as program IOMobileFramebuffer.
# Uses Semeru 21. Never the host macOS dyld cache.
set -euo pipefail

ROOT="${WWN_IOMFB_RE:-$HOME/.vphone/re}"
MACHO="${WWN_IOMFB_MACHO:-$ROOT/guest-iomfb/IOMobileFramebuffer}"
PROJ="${WWN_IOMFB_GHIDRA_PROJ:-$HOME/GhidraVibe/ghidra-vibe-projects/wwn-iomfb}"
JAVA_HOME="${GHIDRA_VIBE_JAVA_HOME:-${JAVA_HOME:-}}"
UTIL="${GHIDRA_VIBE_UTILITY_JAR:-/tmp/ghidra-vibe-runtime-ss2/Ghidra/Framework/Utility/lib/Utility.jar}"

echo "extract root: $ROOT"
echo "mach-o: $MACHO"
echo "project: $PROJ"
if [[ ! -f "$MACHO" ]]; then
  echo "missing Mach-O; run scripts/lab-extract.sh first" >&2
  exit 1
fi
if [[ ! -x "${JAVA_HOME}/bin/java" ]]; then
  echo "set GHIDRA_VIBE_JAVA_HOME to IBM Semeru 21" >&2
  exit 1
fi
if [[ ! -f "$UTIL" ]]; then
  echo "missing Utility.jar: $UTIL" >&2
  exit 1
fi

mkdir -p "$PROJ"
"${JAVA_HOME}/bin/java" \
  -Djava.system.class.loader=ghidra.GhidraClassLoader \
  -Dfile.encoding=UTF8 \
  -Djava.awt.headless=true \
  -Xshare:off \
  --enable-native-access=ALL-UNNAMED \
  -Xmx2G \
  -cp "$UTIL" \
  ghidra.Ghidra ghidra.app.util.headless.AnalyzeHeadless \
  "$PROJ" wwn-iomfb \
  -import "$MACHO" \
  -processor AARCH64:LE:64:v8A \
  -cspec default \
  -noanalysis

echo
echo "program name must be IOMobileFramebuffer"
echo "never import the host macOS dyld cache"
echo "then list_exports / decompile each docs/ABI.md row"
