# wwn-iomfb-rs

MIT reconstruction of iOS **IOMobileFramebuffer** for TrollStore `.tipa`
apps. Safe Rust API plus a C ABI. No Apple headers. No Apple source. No
jailbreak required at runtime.

Reverse engineering uses a jailbroken **vphone** lab (`wawona-jb`, iOS 26.1
/ 23B85). The guest dyld cache is the authority binary. Host macOS DSC,
Simulator, and the public iPhoneOS SDK stub are not.

This is **not** Wawona Desktop Replacement. Wawona Mode B may consume this
crate later. The current Wawona sink (`wwn-iland-iomfb`) is frozen.

## Layer (repo DAG)

**L3′**. nixpkgs-only. Must not become an input of L0-L2. L1 `wwn-iland`
must not import this. Consumer (later): **Wawona** L4. See
`Wawona/docs/wwn-repo-dag.md`.

## Crates

| Crate | Role |
|---|---|
| `iomfb-abi` | Reconstructed types, selectors, claim status |
| `iomfb-sys` | `dlopen` / `dlsym` table |
| `iomfb` | Safe high-level API (grows only when `docs/ABI.md` says confirmed) |
| `iomfb-c` | C ABI for ObjC / Swift tipas |

ObjC in `ffi/` is a trampoline for `CGRect` / `IOSurface` only.

## Docs

- [`docs/SOURCES.md`](docs/SOURCES.md): bibliography. Cite, do not copy.
- [`docs/CLAIMS.md`](docs/CLAIMS.md): every prior claim. Starts unconfirmed.
- [`docs/ABI.md`](docs/ABI.md): our iOS 26.1 guest findings.
- [`docs/LAB.md`](docs/LAB.md): AI loop (vphone, ipsw, GhidraVibe, agent-device).
- [`docs/kernel/README.md`](docs/kernel/README.md): IOConnect evidence. Not linked.

A claim is `confirmed` only after GhidraVibe on the named guest program plus
(for swap / power / restore) a tipa or lab call that returns 0.

## Hard rejects

- Apple headers or Ghidra decompiler paste in git
- IPSW, DSC, `Disk.img`, extracted Mach-O in git
- App Store / Play / Mode A IPA
- ElleKit
- Attach to `watchdogd` / IOWatchdog
- `killall backboardd`

## License

MIT. See [LICENSE](LICENSE).
