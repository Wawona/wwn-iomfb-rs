#!/usr/bin/env bash
# Create the private repo, Discord push webhook, milestones, and claim issues.
# Needs a classic PAT (or a fine-grained token with lifetime <= 366 days)
# that can create repos in the Wawona org. The current keychain token cannot.
set -euo pipefail

REPO="${GITHUB_REPOSITORY:-Wawona/wwn-iomfb-rs}"
HERE="$(cd "$(dirname "$0")/.." && pwd)"

if ! command -v gh >/dev/null; then
  echo "gh not on PATH; try: nix shell nixpkgs#gh -c $0" >&2
  exit 1
fi

if ! gh api repos/Wawona/wwn-iomfb-rs --jq .full_name >/dev/null 2>&1; then
  gh repo create Wawona/wwn-iomfb-rs --private \
    --description "Reconstructed iOS IOMobileFramebuffer (MIT). L3-prime, nixpkgs-only." \
    --source "$HERE" --remote origin --push
fi

URL=$(gh api repos/Wawona/Wawona/hooks --jq \
  '.[] | select((.config.url // "") | endswith("/github")) | .config.url' | head -1)
if [[ -z "$URL" ]]; then
  echo "could not copy Discord webhook URL from Wawona/Wawona" >&2
  exit 1
fi
EXISTING=$(gh api repos/Wawona/wwn-iomfb-rs/hooks --jq \
  --arg u "$URL" '.[] | select(.config.url==$u) | .id' | head -1 || true)
if [[ -z "$EXISTING" ]]; then
  jq -n --arg url "$URL" \
    '{name:"web",active:true,events:["push"],config:{url:$url,content_type:"json",insecure_ssl:"0"}}' \
    | gh api repos/Wawona/wwn-iomfb-rs/hooks --method POST --input -
fi

while IFS='|' read -r title due; do
  [[ -z "$title" ]] && continue
  if gh api "repos/$REPO/milestones" --jq '.[].title' | grep -qx "$title"; then
    continue
  fi
  gh api "repos/$REPO/milestones" --method POST -f title="$title" >/dev/null
  echo "milestone $title"
done <<'MS'
M0 Bootstrap|
M1 Lab extract|
M2 Export census plus claims ingest|
M3 Swap family|
M4 Power and vsync|
M5 Exclusive search|
M6 Safe Rust + C ABI|
M7 Tipa proof|
M8 Remaining exports|
M9 Hand-off|
MS

create_issue() {
  local title="$1"
  local body="$2"
  local mile="$3"
  if gh issue list --repo "$REPO" --search "$title in:title" --json title \
    --jq '.[].title' | grep -qx "$title"; then
    return 0
  fi
  gh issue create --repo "$REPO" --title "$title" --body "$body" --milestone "$mile"
}

create_issue "M0 Bootstrap" "Private repo, MIT, FUNDING, Discord hook, flake, DAG cite." "M0 Bootstrap"
create_issue "M1 Lab extract" "Guest IOMFB Mach-O on host, imported, program named IOMobileFramebuffer." "M1 Lab extract"
create_issue "M2 Export census plus claims ingest" "See docs/CLAIMS.md and docs/ABI.md. Every S1-S5 row starts unconfirmed until verified." "M2 Export census plus claims ingest"
create_issue "M3 Swap family" "Settle S6 first. Confirm GetMain/Secondary, GetDisplaySize, Swap*, GetLayerDefaultSurface." "M3 Swap family"
create_issue "M4 Power and vsync" "EnableDisableVideoPowerSavings polarity, RequestPowerChange, vsync, layer count." "M4 Power and vsync"
create_issue "M5 Exclusive search" "Unused exports that disable other clients. If none, document hold." "M5 Exclusive search"
create_issue "M6 Safe Rust + C ABI" "Confirmed families only." "M6 Safe Rust + C ABI"
create_issue "M7 Tipa proof" "Install, swap frames, restore default surface." "M7 Tipa proof"
create_issue "M8 Remaining exports" "Bind + smoke until the census is 100% confirmed or refuted." "M8 Remaining exports"
create_issue "M9 Hand-off" "Wawona may switch Mode B present to this crate. Not implemented here." "M9 Hand-off"

create_issue "S1 Wawona Mode B claims" "See docs/CLAIMS.md S1. In-use-unverified until guest 26.1 confirms." "M2 Export census plus claims ingest"
create_issue "S2 Public userspace header claims" "Legacy 3-arg SetLayer vs modern 6-arg. See docs/CLAIMS.md S2." "M2 Export census plus claims ingest"
create_issue "S3 Apple Wiki userclient claims" "Selectors and SwapArg. See docs/CLAIMS.md S3." "M3 Swap family"
create_issue "S4 aiaf _kern map (Sonoma lead)" "Lead only. Confirm on iOS 26.1. See docs/CLAIMS.md S4." "M3 Swap family"
create_issue "S5 vphone CFW claims" "Trampoline / struct size / guest patch. See docs/CLAIMS.md S5." "M1 Lab extract"
create_issue "S6 First-pass conflicts" "SetLayer arity, layer count, SwapEnd, size type, polarity, cancel, exclusive." "M3 Swap family"

echo "ok $REPO"
