# wwn-iomfb-rs agent rules

Libre IOMFB reconstruction. Authority: guest vphone `wawona-jb` iOS 26.1
dyld cache image `IOMobileFramebuffer`.

## Do

- Write new logic in Rust. ObjC is `ffi/` trampoline only
  (IOSurface create + Metal wrap). Present policy stays in Rust.
- GPU present is zero-copy IOSurface. Do not add a blit on the
  Desktop path. See `docs/GPU.md`. vphone is the proof device.
- Seed and update `docs/CLAIMS.md` / `docs/ABI.md`. No crate API from an
  unconfirmed row.
- Confirm or refute every prior claim on the guest image (GhidraVibe,
  program name `IOMobileFramebuffer`).
- Run the lab loop in `docs/LAB.md`.
- Keep Discord push webhook + `.github/FUNDING.yml` on this repo.

## Never

- Copy Apple headers, gist bodies, wiki C, or Ghidra pseudocode into git
- Commit DSC / IPSW / Disk.img / extracted Mach-O
- Treat host macOS DSC or Simulator as authority
- Add this flake as an input of L0-L2 (`wwn-toolchain`, `wwn-iland`,
  `wwn-kmscube`)
- Ship in App Store / Play artifacts
- Park `watchdogd` or `killall backboardd`

HID / SpringBoard park is Wawona Desktop policy, not this crate.

## DAG

L3′. nixpkgs-only. See `Wawona/docs/wwn-repo-dag.md`.
