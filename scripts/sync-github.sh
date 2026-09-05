#!/usr/bin/env bash
# Mirror docs/CLAIMS.md + docs/ABI.md into family and milestone issues.
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

WORKDIR="$(mktemp -d)"
trap 'rm -rf "$WORKDIR"' EXIT

python3 - "$CLAIMS" "$ABI" "$WORKDIR" <<'PY'
import pathlib
import re
import sys

claims = pathlib.Path(sys.argv[1]).read_text()
abi = pathlib.Path(sys.argv[2]).read_text()
out = pathlib.Path(sys.argv[3])

def count_status(text: str) -> dict[str, int]:
    return {
        name: len(re.findall(rf"\| {name} \|", text))
        for name in ("unconfirmed", "confirmed", "refuted", "absent")
    }

def section(md: str, key: str) -> str:
    pat = rf"(?ms)^## {re.escape(key)}\b.*?(?=^## |\Z)"
    m = re.search(pat, md)
    return m.group(0).strip() if m else ""

def leftover_rows(sec: str) -> list[str]:
    rows = []
    for line in sec.splitlines():
        if "| unconfirmed |" in line and line.strip().startswith("|"):
            rows.append(line.strip())
    return rows

totals = count_status(claims)
public = 0
m = re.search(r"Public exports \| (\d+)", abi)
if m:
    public = int(m.group(1))

families = {
    "S1": "S1 Wawona Mode B claims",
    "S2": "S2 Public userspace header claims",
    "S3": "S3 Apple Wiki userclient claims",
    "S4": "S4 aiaf _kern map (Sonoma lead)",
    "S5": "S5 vphone CFW claims",
    "S6": "S6 First-pass conflicts",
}

header = (
    "Source of truth: `docs/CLAIMS.md` and `docs/ABI.md`.\n"
    "Do not edit status by hand. Re-run `scripts/sync-github.sh`.\n"
)

for key, title in families.items():
    sec = section(claims, key)
    counts = count_status(sec)
    left = leftover_rows(sec)
    leftover_note = ""
    if left:
        leftover_note = "Leftover unconfirmed (not crate ABI):\n\n" + "\n".join(
            f"- `{line.split('|')[1].strip()}`" for line in left
        ) + "\n\n"
    settled = "yes" if counts["unconfirmed"] == 0 else "no"
    body = (
        f"{header}\n"
        f"Family **{key}**. settled={settled} "
        f"unconfirmed={counts['unconfirmed']} confirmed={counts['confirmed']} "
        f"refuted={counts['refuted']} absent={counts['absent']}\n\n"
        f"{leftover_note}"
        f"{sec}\n"
    )
    (out / f"{key}.md").write_text(body)
    (out / f"{key}.meta").write_text(f"{title}\n{settled}\n")

milestones = {
    "M0 Bootstrap": "done. Private MIT repo, FUNDING, Discord push hook, DAG L3-prime cite.",
    "M1 Lab extract": "done. Guest IOMFB imported as Ghidra program `/IOMobileFramebuffer`.",
    "M2 Export census plus claims ingest": (
        f"done. Public exports={public}. Claim rows: "
        f"unconfirmed={totals['unconfirmed']} confirmed={totals['confirmed']} "
        f"refuted={totals['refuted']} absent={totals['absent']}."
    ),
    "M3 Swap family": "done. S6 settled on guest 26.1. Swap/display/default-surface confirmed.",
    "M4 Power and vsync": "done. Power-save polarity, RequestPowerChange, vsync notify type 5 / sel 0x48.",
    "M5 Exclusive search": "done. No exclusive/disable-others export. Hold is TrollStore policy.",
    "M6 Safe Rust + C ABI": "done. Confirmed families bound in iomfb / iomfb-c. Product path is userspace.",
    "M7 Tipa proof": "done. Smoke + bench tipas on vphone. Userspace bench: userland=1 bound=0 has_metal=1.",
    "M8 Remaining exports": "done. 153/153 public names confirmed. Live return codes in docs/LIVE.md.",
    "M9 Hand-off": "open. Crate ships iomfb-ios / wwn_iomfb_*. S1-iland-bind waits on a Mode B present that returns 0.",
}

for title, blurb in milestones.items():
    state = "open" if title.startswith("M9") else "done"
    body = (
        f"{header}\n"
        f"{title}: {blurb}\n\n"
        f"Claim rows overall: unconfirmed={totals['unconfirmed']} "
        f"confirmed={totals['confirmed']} refuted={totals['refuted']} "
        f"absent={totals['absent']}. Public exports={public}.\n"
    )
    slug = title.split()[0]
    (out / f"{slug}.md").write_text(body)
    (out / f"{slug}.meta").write_text(f"{title}\n{state}\n")
PY

issue_for_title() {
  local title="$1"
  gh issue list --repo "$REPO" --search "$title in:title" --limit 20 \
    --json number,title --jq ".[] | select(.title==\"$title\") | .number" | head -1
}

sync_issue() {
  local title="$1"
  local body_file="$2"
  local number
  number="$(issue_for_title "$title")"
  if [[ -z "$number" ]]; then
    echo "skip (no issue): $title" >&2
    return 0
  fi
  gh issue edit --repo "$REPO" "$number" --body-file "$body_file" >/dev/null
  echo "updated #$number $title" >&2
  printf '%s\n' "$number"
}

close_if_open() {
  local number="$1"
  local title="$2"
  local state
  state="$(gh issue view --repo "$REPO" "$number" --json state --jq .state)"
  if [[ "$state" == "OPEN" ]]; then
    gh issue close --repo "$REPO" "$number" --reason completed \
      --comment "Settled on guest 26.1. Status lives in docs/CLAIMS.md." >/dev/null
    echo "closed #$number $title" >&2
  else
    echo "already closed #$number $title" >&2
  fi
}

for key in S1 S2 S3 S4 S5 S6; do
  title="$(sed -n '1p' "$WORKDIR/$key.meta")"
  settled="$(sed -n '2p' "$WORKDIR/$key.meta")"
  number="$(sync_issue "$title" "$WORKDIR/$key.md")"
  if [[ -n "$number" && "$settled" == "yes" ]]; then
    close_if_open "$number" "$title"
  fi
done

for slug in M0 M1 M2 M3 M4 M5 M6 M7 M8 M9; do
  title="$(sed -n '1p' "$WORKDIR/$slug.meta")"
  state="$(sed -n '2p' "$WORKDIR/$slug.meta")"
  number="$(sync_issue "$title" "$WORKDIR/$slug.md")"
  if [[ -n "$number" && "$state" == "done" ]]; then
    close_if_open "$number" "$title"
  fi
done
