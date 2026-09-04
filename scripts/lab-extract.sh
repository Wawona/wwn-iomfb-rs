#!/usr/bin/env bash
# Copy guest IOMFB out of vphone DSC. Never commit the Mach-O.
set -euo pipefail

ROOT="${WWN_IOMFB_RE:-$HOME/.vphone/re}"
GUEST_DSC="$ROOT/guest-dsc"
GUEST_OUT="$ROOT/guest-iomfb"
IP_FILE="${VPHONE_IP_FILE:-$HOME/.vphone/VMs/wawona-jb/guest-ip.txt}"
TOOLS="${VPHONE_TOOLS:-$HOME/.vphone/src/vphone-cli/.tools/bin}"
export PATH="$TOOLS:$PATH"

if [[ ! -f "$IP_FILE" ]]; then
  echo "missing guest-ip.txt: $IP_FILE" >&2
  exit 1
fi
IP="$(tr -d '[:space:]' < "$IP_FILE")"
if [[ -z "$IP" ]]; then
  echo "empty guest IP" >&2
  exit 1
fi

mkdir -p "$GUEST_DSC" "$GUEST_OUT"
SSHPASS="${SSHPASS:-alpine}"
SSH=(sshpass -e ssh -p 22222 -o StrictHostKeyChecking=no -o ConnectTimeout=8
  "root@$IP")

echo "guest $IP -> $GUEST_DSC"
export SSHPASS
"${SSH[@]}" 'export PATH=/var/jb/usr/bin:/usr/bin:$PATH
  ls -lh /System/Library/Caches/com.apple.dyld/dyld_shared_cache_arm64e*'

# Framework dir is a stub. Guest sh does not expand dyld_shared_cache_arm64e.*.
# List first, then tar every slice (about 5.6G).
"${SSH[@]}" 'export PATH=/var/jb/usr/bin:/usr/bin:$PATH
  cd /System/Library/Caches/com.apple.dyld
  files=$(ls dyld_shared_cache_arm64e*)
  echo FILES:$(echo $files | wc -w) >&2
  tar cf - $files' \
  > "$GUEST_DSC/guest-dsc.tar"

tar -C "$GUEST_DSC" -xf "$GUEST_DSC/guest-dsc.tar"
MAIN="$(ls "$GUEST_DSC"/dyld_shared_cache_arm64e 2>/dev/null || true)"
if [[ -z "$MAIN" ]]; then
  echo "no dyld_shared_cache_arm64e in $GUEST_DSC" >&2
  exit 1
fi

if ! command -v ipsw >/dev/null; then
  echo "ipsw not on PATH; cache copied to $GUEST_DSC" >&2
  echo "run: ipsw dyld extract $MAIN --output $GUEST_OUT IOMobileFramebuffer" >&2
  exit 0
fi

ipsw dyld extract "$MAIN" --output "$GUEST_OUT" IOMobileFramebuffer || {
  echo "ipsw extract failed; try: ipsw dyld extract $MAIN IOMobileFramebuffer" >&2
  exit 1
}

echo "extracted under $GUEST_OUT"
find "$GUEST_OUT" -iname '*IOMobileFramebuffer*' -type f | head
