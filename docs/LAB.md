# Lab runbook (AI loop)

One documented loop. Any agent can run it. MCP plus scripts, not a new
daemon.

```text
vphone wawona-jb
  -> extract guest IOMFB (or same-build IPSW cross-check)
  -> GhidraVibe program IOMobileFramebuffer
  -> docs/ABI.md
  -> iomfb crate (confirmed rows only)
  -> TrollStore tipa
  -> GitHub issues via scripts/sync-github.sh
```

## 1. Lab up

```bash
# VM window must be visible for sock screenshots
open ~/.vphone/src/vphone-cli/build/Release/vphone-cli.app --args \
  --config ~/.vphone/VMs/wawona-jb/config.plist --variant jb

agent-device --version   # 0.18.3-wawona.N+
agent-device devices     # vphone wawona-jb booted=true
```

Guest SSH: `root` or `mobile` / `alpine`, port `22222`. IP from
`~/.vphone/VMs/wawona-jb/guest-ip.txt` (drifts). Guest has no
`/bin/cat` or `/bin/ls`. Use `/var/jb/usr/bin/*`. No sftp: push files
with `ssh … 'dd of=…' < file`.

Jailbreak RE packages (Procursus): `debugserver` (lab bootstrap),
`odcctools` (`nm`, `otool`), `llvm-14` / `llvm-16`. Frida is in
`frida.list` if needed. Host `ipsw macho disass --force` on the
extracted Mach-O is the fast arity pass. vphone ships
`Metal.framework`. The Metal tipa is the GPU proof.

```bash
export PATH="$HOME/.vphone/src/vphone-cli/.tools/bin:$PATH"
SSHPASS=alpine sshpass -e ssh -p 22222 \
  -o StrictHostKeyChecking=no root@$(cat ~/.vphone/VMs/wawona-jb/guest-ip.txt) \
  'export PATH=/var/jb/usr/bin:/usr/bin:$PATH; uname -a'
```

GhidraVibe: `vibe_health`. Analysis MCP on `:8089`. Always pass program
name `IOMobileFramebuffer`. Open Bluetooth / Wi-Fi programs are the wrong
target.

Never attach to `watchdogd`. Never `killall backboardd`.

Sock screenshots need the visible `vphone-cli.app` window. Digitizer unlock
uses **full-res** coords (hostctl default, e.g. 1290x2796), not the compact
JPEG size.

Session name: `vphone`. Device name: `vphone wawona-jb`.

## 2. Extract

```bash
./scripts/lab-extract.sh
```

Copies guest dyld cache slices to `~/.vphone/re/` (outside git). Then:

```bash
ipsw dyld extract ~/.vphone/re/guest-dsc/dyld_shared_cache_arm64e \
  --output ~/.vphone/re/guest-iomfb \
  IOMobileFramebuffer
```

Same-build research IPSW is `iPhone99,11` / 23B85 (this guest class).
`iPhone17,3` is a different SoC. Never commit the Mach-O.

Diff guest vs stock before treating guest bytes as Apple:

```bash
cmp -l ~/.vphone/re/guest-iomfb/IOMobileFramebuffer \
       ~/.vphone/re/stock-23B85/IOMobileFramebuffer | head
```

## 3. Import

Program name **must** be `IOMobileFramebuffer`. Project path must **not**
contain a dotted directory (Ghidra rejects `.vphone`). Use Semeru 21
directly. Darwin HotSpot SIGBUS in `CodeHeap::allocate`. `launch.sh` JDK
prompt needs a TTY; skip it:

```bash
JAVA="$GHIDRA_VIBE_JAVA_HOME/bin/java"
UTIL=/tmp/ghidra-vibe-runtime-ss2/Ghidra/Framework/Utility/lib/Utility.jar
PROJ=$HOME/GhidraVibe/ghidra-vibe-projects/wwn-iomfb
"$JAVA" -Djava.system.class.loader=ghidra.GhidraClassLoader \
  -Djava.awt.headless=true -Xshare:off -Xmx2G -cp "$UTIL" \
  ghidra.Ghidra ghidra.app.util.headless.AnalyzeHeadless \
  "$PROJ" wwn-iomfb \
  -import ~/.vphone/re/guest-iomfb/IOMobileFramebuffer \
  -processor AARCH64:LE:64:v8A -cspec default -noanalysis
```

`user-ghidra` `import_file` on the SS2 `:8089` server is GUI-only and
currently has Bluetooth / Wi-Fi open. `dyld_find_cache` / default
`dyld_import_image` see the **host** macOS cache. Do not import that.

## 4. Census

```bash
ipsw dyld symaddr --image IOMobileFramebuffer \
  ~/.vphone/re/guest-dsc/dyld_shared_cache_arm64e
```

Plus Ghidra `list_exports` / `search_functions`. Seed every
`IOMobileFramebuffer*` and `_kern_*` / `_virt_*` row in `docs/ABI.md`.

## 5. Confirm

For each symbol: decompile, record arity / `CGRect` passing / IOConnect
selector, mark confirmed or refuted, implement or drop. One family per PR.

S6 first: SetLayer arity, SwapEnd shape, layer count, size type,
power-save polarity, cancel selector, exclusive export.

## 6. Prove

Slim TrollStore tipa. ldid IOMFB ents. Install into
`/var/containers/Bundle/Application/…`, never `/var/jb/Applications/`.

```bash
packages tipa install ./….tipa --device "vphone wawona-jb"
packages tipa open-jit <bundle> --device "vphone wawona-jb"
```

Sock screenshot plus logs. No SpringBoard park in this crate. Exclusive is
an IOMFB export or a documented "none exists" result.

Metal tipa: `scripts/build-tipa-metal.sh`. Jailbreak CLI:
`scripts/build-jb-cli.sh`, then `dd` to `/var/jb/usr/local/bin/iomfb-cli`.
No ElleKit.

## 7. Sync

```bash
./scripts/sync-github.sh
```

Updates milestone issues from `docs/ABI.md` checkboxes. No manual drift.

## MCP notes

- `user-ghidra`: `import_file`, `list_exports`, `decompile_function`,
  `get_function_signature`. Always `switch_program` to
  `IOMobileFramebuffer`.
- `user-ghidra-vibe`: `vibe_health`, `dyld_import_image` (guest cache
  only), `malimite_*` if a tipa is under study.
- `user-agent-device`: `devices`, `snapshot`, `press`, `screenshot`.
  Prefer device name `vphone wawona-jb`.
- `user-lldb`: never attach to `watchdogd`.
