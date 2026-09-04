# wwn-iomfb-rs

MIT reconstruction of iOS **IOMobileFramebuffer**. Safe Rust, C ABI, and
a Metal/IOSurface GPU path. No Apple IOMFB headers. No Apple source.

**Zero-copy is the product.** Metal renders into an IOSurface. IOMFB
swaps that same surface. That is the Apple dma-buf. See [`docs/GPU.md`](docs/GPU.md).

Reverse engineering uses jailbroken **vphone** `wawona-jb` (iOS 26.1 /
23B85). The guest dyld cache is the authority binary.

This crate is the present library for:

- TrollStore framebuffer apps (Metal + touch mapping)
- Jailbreak tweaks that own the panel
- Later **Wawona Mode B Desktop Replacement** (TrollStore or Sileo)

It is not Desktop Replacement itself. HID / SpringBoard park stays in
Wawona. The frozen sink `wwn-iland-iomfb` must not grow ABI guesses.

## Layer (repo DAG)

**L3′**. nixpkgs-only. L1 `wwn-iland` must not import this. Consumer:
**Wawona** L4. See `Wawona/docs/wwn-repo-dag.md`.

## Crates

| Crate | Role |
|---|---|
| `iomfb-abi` | Types, selectors, GPU present status |
| `iomfb-sys` | `dlopen` / `dlsym` table |
| `iomfb` | Safe API: display, swap, [`GpuSwapchain`](crates/iomfb/src/gpu.rs), [`TouchMap`](crates/iomfb/src/touch.rs) |
| `iomfb-c` | C ABI (`include/iomfb.h`) for ObjC / Swift / tweaks |

ObjC in `ffi/` creates IOSurfaces and wraps Metal textures. Policy stays
in Rust.

## Present (Wawona Desktop hook)

```rust
let mut gpu = iomfb::GpuSwapchain::main()?;
let frame = gpu.acquire()?;
// Metal: draw into frame.metal_texture (same IOSurface as frame.surface)
let status = gpu.present()?;
assert!(status.zero_copy);

// Compositor already has an IOSurface (linux-dmabuf import):
gpu.present_external(wayland_iosurface)?;
```

TrollStore and tweaks call the same entry points through `include/iomfb.h`.

## Docs

- [`docs/GPU.md`](docs/GPU.md): zero-copy, consumers, evidence
- [`docs/SOURCES.md`](docs/SOURCES.md): bibliography. Cite, do not copy.
- [`docs/CLAIMS.md`](docs/CLAIMS.md): prior claims.
- [`docs/ABI.md`](docs/ABI.md): iOS 26.1 guest findings.
- [`docs/LAB.md`](docs/LAB.md): vphone / GhidraVibe loop.

## Hard rejects

- Apple IOMFB headers or Ghidra decompiler paste
- IPSW, DSC, `Disk.img`, extracted Mach-O in git
- App Store / Play / Mode A IPA
- ElleKit inside this crate
- CPU blit on the Desktop present path
- Attach to `watchdogd` / `killall backboardd`

## License

MIT. See [LICENSE](LICENSE).
