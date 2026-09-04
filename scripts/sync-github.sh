#!/usr/bin/env bash
# Mirror docs/ABI.md + docs/CLAIMS.md into milestone issues.
# Requires gh authenticated to github.com/Wawona/wwn-iomfb-rs.
set -euo pipefail

REPO="${GITHUB_REPOSITORY:-Wawona/wwn-iomfb-rs}"
HERE="$(cd "$(dirname "$0")/.." && pwd)"
ABI="$HERE/docs/ABI.md"
CLAIMS="$HERE/docs/CLAIMS.md"

if ! command -v gh >/dev/null; then
  echo "gh not on PATH; try: nix shell nixpkgs#gh -c $0" >&2
  exit 1
fi

if [[ ! -f "$ABI" || ! -f "$CLAIMS" ]]; then
  echo "missing $ABI or $CLAIMS" >&2
  exit 1
fi

unconfirmed=$(grep -c '| unconfirmed |' "$CLAIMS" || true)
confirmed=$(grep -c '| confirmed |' "$CLAIMS" || true)
refuted=$(grep -c '| refuted |' "$CLAIMS" || true)
absent=$(grep -c '| absent |' "$CLAIMS" || true)

body=$(cat <<EOF
Source of truth: \`docs/ABI.md\` and \`docs/CLAIMS.md\`. Do not edit this
issue by hand for status. Re-run \`scripts/sync-github.sh\`.

Claim rows: unconfirmed=$unconfirmed confirmed=$confirmed refuted=$refuted absent=$absent

EOF
)

# One tracking issue per milestone title prefix.
while IFS= read -r title; do
  [[ -z "$title" ]] && continue
  number=$(gh issue list --repo "$REPO" --search "$title in:title" --json number,title \
    --jq ".[] | select(.title==\"$title\") | .number" | head -1)
  if [[ -z "$number" ]]; then
    echo "skip (no issue): $title" >&2
    continue
  fi
  gh issue comment --repo "$REPO" "$number" --body "$body"
  echo "updated #$number $title"
done <<'TITLES'
M0 Bootstrap
M1 Lab extract
M2 Export census plus claims ingest
M3 Swap family
M4 Power and vsync
M5 Exclusive search
M6 Safe Rust + C ABI
M7 Tipa proof
M8 Remaining exports
M9 Hand-off
TITLES
